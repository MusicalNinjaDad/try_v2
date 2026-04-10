use proc_macro2_diagnostic::prelude::*;
use quote::format_ident;
use syn::{
    AngleBracketedGenericArguments, Arm, Data, DataEnum, DeriveInput, Fields, GenericArgument,
    GenericParam, Generics, Ident, ImplGenerics, Lifetime, PathArguments, Type, TypeGenerics,
    TypeNever, TypePath, Variant, WhereClause, parse_quote, punctuated::IntoIter, spanned::Spanned,
};

/// A destructured Enum with validated invariants and easy access to all the bits we need.
pub(crate) struct TryEnum<'ast> {
    name: &'ast Ident,
    enum_data: &'ast DataEnum,
    output_variant_name: &'ast Ident,
    output_type: OutputType<'ast>,
    residual_type: Type,
    generics: &'ast Generics,
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
                    let first_output_usage = fields.unnamed.iter().find_map(|field| {
                        OutputType::try_from((&field.ty, first_generic_type)).ok()
                    });
                    let msg = match first_output_usage {
                        None | Some(OutputType::Owned { .. }) => {
                            format!("change this to ({first_generic_type})")
                        }
                        Some(OutputType::Ref { lifetime, .. }) => {
                            format!("change this to (&{lifetime} {first_generic_type})")
                        }
                        Some(OutputType::Contained { .. }) => todo!("74"),
                    };
                    error("Try requires a single generic type for `Output`")
                        // TODO: Check that multiline enum defs show whole def in help
                        .add_help(first_generic_type.span(), "Output type defined here")
                        .add_help(fields.span(), msg)
                }
                Fields::Unit => error("Try requires a generic type for `Output`")
                    .add_help(first_generic_type.span(), "Output type defined here")
                    .add_help(
                        output_variant.span(),
                        format_args!("add ({first_generic_type}) after this..."),
                    ),
                Fields::Named(fields) => {
                    error("Try requires an unnamed field for the `Output` variant")
                        .add_help(first_generic_type.span(), "Output type defined here")
                        .add_help(
                            fields.span(),
                            format_args!("change this to ({first_generic_type})"),
                        )
                }
            };
        };

        let output_type = OutputType::try_from((output_type, first_generic_type))?;

        // Must be done late, after validating other invariants
        let residual_type: Type = generate_residual(ast);

        Ok(Self {
            name,
            enum_data,
            output_variant_name,
            output_type,
            residual_type,
            generics: &ast.generics,
        })
    }

    /// Create match arms for `fn branch` and `fn from_residual`.
    pub(crate) fn generate_arms(&self) -> (Vec<BranchArm>, Vec<Option<ResidualArm>>) {
        let enum_name: &Ident = self.name;

        let arms = |variant: &Variant| -> (BranchArm, Option<ResidualArm>) {
            let var_name: &Ident = &variant.ident;

            match &variant.fields {
                _ if variant.ident == *self.output_variant_name => {
                    // Relies on invariant: Output variant always has a single field
                    let branch_arm = parse_quote! {
                        Self::#var_name(v0) => std::ops::ControlFlow::Continue(v0),
                    };

                    let residual_arm = match self.output_type {
                        OutputType::Owned { .. } => None,
                        OutputType::Ref { .. } => {
                            // &! is not recognised as infallible, but ! will coerce to any other type.
                            // - see https://github.com/rust-lang/unsafe-code-guidelines/issues/413
                            // - and https://users.rust-lang.org/t/whats-the-right-syntax-for-an-infallible-reference/139188
                            Some(parse_quote! {
                                #enum_name::#var_name(never) => *never,
                            })
                        }
                        // TODO: #53 Find a way for compiler to verify Box<!>, Vec<!> etc is impossible
                        OutputType::Contained { .. } => Some(parse_quote! {
                            #enum_name::#var_name(foo) => unreachable!("Container of !"),
                        }),
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

        self.enum_data.variants.iter().map(arms).unzip()
    }

    /// Provides a cloned set of generics after applying `change`
    ///
    /// `generics()` is designed for situations where you can modify the generics in place, use
    /// `generics_with_params()` where the parameters need redefining in a way which can be better
    /// achieved via `.into_iter()....collect()`
    pub(crate) fn generics<C>(&self, mut change: C) -> Generics
    where
        C: FnMut(&mut Generics),
    {
        let mut generics = self.generics.clone();
        change(&mut generics);
        generics
    }

    /// Provides a cloned set of generics after applying the `adaptor` to the params.
    ///
    /// `generics_with_params()` is designed for situations where the parameters need redefining
    /// in a way which can be best achieved via `.into_iter()....collect()`, use `generics()`
    /// for situations where you can modify the generics in place.
    pub(crate) fn generics_with_params<P, I>(&self, adaptor: P) -> Generics
    where
        P: FnOnce(IntoIter<GenericParam>) -> I,
        I: Iterator<Item = GenericParam>,
    {
        let mut generics = self.generics.clone();
        generics.params = adaptor(generics.params.into_iter()).collect();
        generics
    }

    pub(crate) fn split_for_impl(
        &'ast self,
    ) -> (
        &'ast Name,
        &'ast OutputVariantName,
        &'ast OutputTypeTy,
        &'ast OutputTypeName,
        &'ast ResidualType,
        ImplGenerics<'ast>,
        TypeGenerics<'ast>,
        Option<&'ast WhereClause>,
    ) {
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        (
            self.name,
            self.output_variant_name,
            self.output_type.ty(),
            self.output_type.name(),
            &self.residual_type,
            impl_generics,
            ty_generics,
            where_clause,
        )
    }
}

// Helper type aliases to improve function signatures (understandable return values)
type Name = Ident;
type OutputVariantName = Ident;
type OutputTypeTy = Type;
type OutputTypeName = Ident;
type ResidualType = Type;
type BranchArm = Arm;
type ResidualArm = Arm;

/// A Valid Type for an output variant is either a single Ident, or a reference to a single Ident.
/// Invariant validation is **NOT** managed here and should be ensured by any code which produces
/// an `OutputType`
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
    Contained {
        name: &'ast Ident,
        container: &'ast TypePath,
        ty: &'ast Type,
    },
}

impl<'ast> OutputType<'ast> {
    fn name(&self) -> &'ast Ident {
        match self {
            Self::Owned { name, .. } | Self::Ref { name, .. } | Self::Contained { name, .. } => {
                name
            }
        }
    }

    fn ty(&self) -> &'ast Type {
        match self {
            Self::Owned { ty, .. } | Self::Ref { ty, .. } | Self::Contained { ty, .. } => ty,
        }
    }
}

impl<'ast> TryFrom<(&'ast Type, &'ast Ident)> for OutputType<'ast> {
    type Error = DiagnosticResult<!>;

    fn try_from((ty, first_generic_type): (&'ast Type, &'ast Ident)) -> Result<Self, Self::Error> {
        let base_error = || -> DiagnosticResult<!> {
            error("Try requires the first generic type to be used as the `Output` type")
                .add_help(first_generic_type.span(), "Output type defined here")
        };

        let checked_name = |t: &'ast Type| -> Option<&'ast Ident> {
            if let Type::Path(tp) = t {
                tp.path
                    .get_ident()
                    .filter(|ident| *ident == first_generic_type)
            } else {
                None
            }
        };

        // TODO: #47 handle Vec<T>, &[T], Box<T> etc...
        match ty {
            Type::Path(type_path)
                if let PathArguments::None = type_path
                    .path
                    .segments
                    .last()
                    .expect("path has at least one segment")
                    .arguments =>
            {
                Result::Ok(Self::Owned {
                    name: checked_name(ty).ok_or_else(|| {
                        base_error().add_help(
                            ty.span(),
                            format_args!("change this to {first_generic_type}"),
                        )
                    })?,
                    ty,
                })
            }
            Type::Path(type_path) => {
                let last_segment = type_path
                    .path
                    .segments
                    .last()
                    .expect("path has at least one segment");
                if let PathArguments::AngleBracketed(args) = &last_segment.arguments
                    && args.args.len() == 1
                    && let GenericArgument::Type(generic_type) =
                        args.args.first().expect("args.args.len() == 1")
                {
                    let container = type_path;
                    let name = checked_name(generic_type).ok_or_else(|| {
                        base_error().add_help(
                            generic_type.span(),
                            format_args!("change this to {first_generic_type}"),
                        )
                    })?;
                    Result::Ok(Self::Contained {
                        name,
                        container,
                        ty,
                    })
                } else {
                    todo!("Very wrong path, or too many generics")
                }
            }
            Type::Reference(tr) => {
                let lifetime = tr
                    .lifetime
                    .as_ref()
                    .expect("References in enum definitions require a specified lifetime");
                let name = checked_name(tr.elem.as_ref()).ok_or_else(|| {
                    base_error().add_help(
                        ty.span(),
                        format_args!("change this to &{lifetime} {first_generic_type}"),
                    )
                })?;
                Result::Ok(Self::Ref { name, ty, lifetime })
            }
            _ => Result::Err(base_error().add_help(
                ty.span(),
                format_args!("change this to {first_generic_type}"),
            )),
        }
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
