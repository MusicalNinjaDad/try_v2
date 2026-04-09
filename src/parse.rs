use proc_macro2_diagnostic::prelude::*;
use quote::format_ident;
use syn::{
    AngleBracketedGenericArguments, Arm, Data, DataEnum, DeriveInput, Fields, GenericArgument,
    Ident, Lifetime, Type, Variant, parse_quote, spanned::Spanned,
};

/// A destructured Enum with validated invariants and easy access to all the bits we need.
pub(crate) struct TryEnum<'ast> {
    name: &'ast Ident,
    enum_data: &'ast DataEnum,
    output_variant_name: &'ast Ident,
    output_type: OutputType<'ast>,
    residual_type: Type,
}

/// An Arm to be used when matching for `fn branch`.
/// Own type for clarity when returning (BranchArm, ResidualArm) from a single function.
type BranchArm = Arm;
/// An Arm to be used when matching for `fn from_residual`.
/// Own type for clarity of when returning (BranchArm, ResidualArm) from a single function.
type ResidualArm = Arm;

/// A Valid Type for an output variant is either a single Ident, or a reference to a single Ident.
/// Invariant validation is **NOT** managed here and should be ensured by any code which produces
/// an `OutputType`
#[allow(unused)]
enum OutputType<'ast> {
    Owned {
        name: &'ast Ident,
        ty: &'ast Type,
    },
    Ref {
        name: &'ast Ident,
        ty: &'ast Type,
        lifetime: &'ast Lifetime,
    },
}

impl<'ast> OutputType<'ast> {
    fn name(&self) -> &'ast Ident {
        match self {
            Self::Owned { name, .. } | Self::Ref { name, .. } => name,
        }
    }

    fn ty(&self) -> &'ast Type {
        match self {
            Self::Owned { ty, .. } | Self::Ref { ty, .. } => ty,
        }
    }
}

impl<'ast> TryFrom<(&'ast Type, &'ast Ident)> for OutputType<'ast> {
    type Error = DiagnosticResult<!>;

    fn try_from((ty, first_generic_type): (&'ast Type, &'ast Ident)) -> Result<Self, Self::Error> {
        match ty {
            Type::Path(type_path) => Result::Ok(Self::Owned {
                name: type_path
                    .path
                    .get_ident()
                    .filter(|ident| *ident == first_generic_type)
                    .ok_or_else(|| {
                        error("Try requires the first generic type to be used as the `Output` type")
                            .add_help(first_generic_type.span(), "Output type defined here")
                            .add_help(
                                ty.span(),
                                format_args!("change this to {first_generic_type}"),
                            )
                    })?,
                ty,
            }),
            Type::Reference(tr) => {
                let lifetime = tr
                    .lifetime
                    .as_ref()
                    .expect("References in enum definitions require a specified lifetime");
                let name = if let Type::Path(tp) = tr.elem.as_ref() {
                    tp.path.get_ident().filter(|ident| *ident == first_generic_type).ok_or_else(|| {
                        error("Try requires the first generic type to be used as the `Output` type")
                            .add_help(first_generic_type.span(), "Output type defined here")
                            .add_help(
                        ty.span(),
                        format_args!(
                            "change this to &{} {first_generic_type}",
                            tr.lifetime.as_ref().expect("generic ref must have lifetime")
                        ))
                    })?
                } else {
                    todo!("ref to invalid type")
                };
                Result::Ok(Self::Ref { name, ty, lifetime })
            }
            _ => Result::Err(
                error("Try requires the first generic type to be used as the `Output` type")
                    .add_help(first_generic_type.span(), "Output type defined here")
                    .add_help(
                        ty.span(),
                        format_args!("change this to {first_generic_type}"),
                    ),
            ),
        }
    }
}

impl<'ast> TryEnum<'ast> {
    /// Handles all the invariant validation and enum un-nesting.
    pub(crate) fn parse(ast: &'ast DeriveInput) -> DiagnosticResult<Self> {
        // Fail fast
        let enum_data: &DataEnum = match &ast.data {
            Data::Enum(enum_data) => Ok(enum_data),
            Data::Struct(struct_data) => error("Try can only be derived for an enum")
                .add_help(struct_data.struct_token.span(), "not an enum"),
            Data::Union(union_data) => error("Try can only be derived for an enum")
                .add_help(union_data.union_token.span(), "not an enum"),
        }?;

        let name: &Ident = &ast.ident;

        let output_variant = enum_data.variants.first().ok_or(
            error("Try cannot be derived for a zero-field enum").add_help(
                enum_data.brace_token.span.span(),
                "add at least two variants here...",
            ),
        )?;
        let output_variant_name: &Ident = &output_variant.ident;

        let first_generic_type: &Ident = ast
            .generics
            .type_params()
            .map(|ty| &ty.ident)
            .next()
            .ok_or(
                error("Try requires a generic type for `Output`")
                    .add_help(name.span(), "Add <T> after this..."),
            )?;

        let output_type = if let Fields::Unnamed(fields) = &output_variant.fields
            && fields.unnamed.len() == 1
        {
            &fields
                .unnamed
                .first()
                .expect("fields.unnamed.len() == 1")
                .ty
        } else {
            return match &output_variant.fields {
                Fields::Unnamed(fields) => {
                    let base_error = error("Try requires a single generic type for `Output`")
                        // TODO: Check that multiline enum defs show whole def in help
                        .add_help(first_generic_type.span(), "Output type defined here");
                    let first_output_usage = &fields
                        .unnamed
                        .iter()
                        .find_map(|field| {
                            OutputType::try_from((&field.ty, first_generic_type)).ok()
                        })
                        .ok_or_else(|| {
                            error("Try requires a single generic type for `Output`")
                                .add_help(first_generic_type.span(), "Output type defined here")
                                .add_help(
                                    fields.span(),
                                    format_args!("change this to ({first_generic_type})"),
                                )
                        })?;
                    match first_output_usage {
                        OutputType::Owned { .. } => base_error.add_help(
                            fields.span(),
                            format_args!("change this to ({first_generic_type})"),
                        ),
                        OutputType::Ref { lifetime, .. } => base_error.add_help(
                            fields.span(),
                            format_args!("change this to (&{lifetime} {first_generic_type})"),
                        ),
                    }
                }
                Fields::Unit => error("Try requires a generic type for `Output`").add_help(
                    output_variant.span(),
                    format_args!("add ({first_generic_type}) after this..."),
                ),
                Fields::Named(fields) => {
                    error("Try requires an unnamed field for the `Output` variant").add_help(
                        fields.span(),
                        format_args!("change this to ({first_generic_type})"),
                    )
                }
            };
        };

        let output_type = OutputType::try_from((output_type, first_generic_type))?;

        // Must be done late, after validating suitable generics
        let residual_type: Type = generate_residual(ast);

        Ok(Self {
            name,
            enum_data,
            output_variant_name,
            output_type,
            residual_type,
        })
    }

    /// ```ignore
    /// let (
    ///     name,
    ///     enum_data,
    ///     output_variant_name,
    ///     output_type,
    ///     output_type_name,
    ///     residual_type,
    /// ) = tryenum.split_for_impl();
    /// ```
    pub(crate) fn split_for_impl(
        &'ast self,
    ) -> (
        &'ast Ident,
        &'ast DataEnum,
        &'ast Ident,
        &'ast Type,
        &'ast Ident,
        &'ast Type,
    ) {
        (
            self.name,
            self.enum_data,
            self.output_variant_name,
            self.output_type.ty(),
            self.output_type.name(),
            &self.residual_type,
        )
    }
}

/// Generate the residual type with appropriate arguments (! + remaining generics).
///
/// Does not act on `self` as this is designed to be called during creation of a `TryEnum`
/// and is only a separate function to facilitate direct testing
///
/// ### Panics
/// if called on unsuitable input, or where invariants (at least one generic type)
/// are not upheld.
fn generate_residual(ast: &DeriveInput) -> Type {
    let name = &ast.ident;
    let (_, ty_generics, _) = ast.generics.split_for_impl();
    let mut typeargs: AngleBracketedGenericArguments = parse_quote!(#ty_generics);
    let first_type = typeargs
        .args
        .iter_mut()
        .find_map(|arg| {
            if let GenericArgument::Type(typ) = arg {
                Some(typ)
            } else {
                None
            }
        })
        .expect("must have at least one generic output type");
    *first_type = parse_quote!(!);
    parse_quote! {#name #typeargs} // e.g. `Foo<!,E,U>`
}

/// Create match arms for `fn branch` and `fn from_residual`.
///
/// Does not act on `TryEnum` as we expect a TryEnum to be immediately destructured and not stored.
pub(crate) fn generate_arms(
    enum_name: &Ident,
    enum_data: &DataEnum,
    output_type: &Type,
) -> (Vec<BranchArm>, Vec<Option<ResidualArm>>) {
    let owned_output = matches!(output_type, Type::Path(_));
    let arms = |(i, variant): (usize, &Variant)| -> (BranchArm, Option<ResidualArm>) {
        let var_name: &Ident = &variant.ident;
        let is_output_variant = i == 0;
        match &variant.fields {
            _ if is_output_variant => {
                // Output variant always has a single field
                let branch_arm = parse_quote! {
                    Self::#var_name(v0) => std::ops::ControlFlow::Continue(v0),
                };
                let residual_arm = if owned_output {
                    None
                } else {
                    // required for when Output stores a reference.
                    // &! is not recognised as infallible, but ! will coerce to any other type.
                    // - see https://github.com/rust-lang/unsafe-code-guidelines/issues/413
                    // - and https://users.rust-lang.org/t/whats-the-right-syntax-for-an-infallible-reference/139188
                    Some(parse_quote! {
                        #enum_name::#var_name(never) => *never,
                    })
                };
                (branch_arm, residual_arm)
            }
            Fields::Unit => {
                let branch_arm = parse_quote! {
                    Self::#var_name => std::ops::ControlFlow::Break(#enum_name::#var_name),
                };
                let residual_arm = parse_quote! {
                    #enum_name::#var_name => #enum_name::#var_name,
                };
                (branch_arm, Some(residual_arm))
            }
            Fields::Unnamed(_) => {
                let fields: Vec<Ident> = (0..variant.fields.len())
                    .map(|n| format_ident!("v{n}"))
                    .collect();
                let branch_arm = parse_quote! {
                    Self::#var_name(#(#fields),*) => std::ops::ControlFlow::Break(#enum_name::#var_name(#(#fields),*)),
                };
                let residual_arm = parse_quote! {
                    #enum_name::#var_name(#(#fields),*) => #enum_name::#var_name(#(#fields),*),
                };
                (branch_arm, Some(residual_arm))
            }
            Fields::Named(_) => {
                let fields: Vec<Ident> = variant
                    .fields
                    .iter()
                    .map(|f| f.ident.clone().expect("named field"))
                    .collect();
                let branch_arm = parse_quote! {
                    Self::#var_name{#(#fields),*} => std::ops::ControlFlow::Break(#enum_name::#var_name{#(#fields),*}),
                };
                let residual_arm = parse_quote! {
                    #enum_name::#var_name{#(#fields),*} => #enum_name::#var_name{#(#fields),*},
                };
                (branch_arm, Some(residual_arm))
            }
        }
    };

    enum_data.variants.iter().enumerate().map(arms).unzip()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum Exit<T> {
                Ok(T),
                TestsFailed,
            }
        };
        let residual = generate_residual(&original);
        let expected_residual: Type = parse_quote! {Exit<!>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn multiple_generics_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum Exit<T, E> {
                Ok(T),
                TestsFailed(E),
            }
        };
        let residual = generate_residual(&original);
        let expected_residual: Type = parse_quote! {Exit<!, E>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn static_ref_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum MyResult<T: 'static, E> {
                Ok(&'static T),
                Err(E),
            }
        };
        let residual = generate_residual(&original);
        let expected_residual: Type = parse_quote! {MyResult<!, E>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn lifetime_ref_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum MyResult<'r, T, E> {
                Ok(&'r T),
                Err(&'r E),
            }
        };
        let residual = generate_residual(&original);
        let expected_residual: Type = parse_quote! {MyResult<'r, !, E>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn multiple_lifetimes_ref_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum MyResult<'t, 'e, T, E> {
                Ok(&'t T),
                Err(&'e E),
            }
        };
        let residual = generate_residual(&original);
        let expected_residual: Type = parse_quote! {MyResult<'t, 'e, !, E>};
        assert_eq!(expected_residual, residual);
    }
}
