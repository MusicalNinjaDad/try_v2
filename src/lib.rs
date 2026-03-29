#![feature(if_let_guard)]
#![feature(never_type)]
#![feature(proc_macro_diagnostic)]
#![feature(try_trait_v2)]

//! Provides a derive macro for `Try`
//! ([try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html))
//!
//! Also enables auto-conversion from `Result<T, E> where E: Into::into(Self)`
//! and back `where Self<!>: Into::into(E)`
//!
//! ## Requires:
//!   - `RUSTC_BOOTSTRAP = 1` (or nightly)
//!   - `#![feature(never_type)]`
//!   - `#![feature(try_trait_v2)]`
//!
//! ## Current Limitations on the annotated type:
//!   - must be an `enum`
//!   - must have _at least one_ generic type
//!   - the _first_ generic type must be the `Output` type (produced when not short circuiting)
//!   - the output variant (does not short-circuit) must be the _first_ variant and store the output
//!     type as the _only unnamed_ field
//!
//! ## Example Usage:
//! ```rust
//! #![feature(never_type)]
//! #![feature(try_trait_v2)]
//! use try_v2::{Try, Try_ConvertResult};
//!
//! #[derive(Try, Try_ConvertResult)]
//! enum TestResult<T> {
//!     Ok(T),
//!     TestsFailed,
//!     OtherError(String)
//! }
//!
//! fn run_tests() -> TestResult<()> {
//!     TestResult::OtherError("oops!".to_string())?; // <- Function short-circuits here ...
//!     TestResult::TestsFailed?;
//!     TestResult::Ok(())
//! }
//!
//! assert!(matches!(run_tests(), TestResult::OtherError(msg) if msg == "oops!"));
//!
//! struct MyError {}
//!
//! impl<T> From<MyError> for TestResult<T> {
//!     fn from(err: MyError) -> Self {
//!         TestResult::TestsFailed
//!     }
//! }
//!
//! fn run_more_tests() -> TestResult<()> {
//!     Err(MyError{})?; // <- Function short-circuits here & converts to a TestResult...
//!     TestResult::Ok(())
//! }
//!
//! assert!(matches!(run_more_tests(), TestResult::TestsFailed));
//! ```

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    AngleBracketedGenericArguments, Arm, Data, DataEnum, DeriveInput, Fields, FieldsUnnamed,
    GenericArgument, GenericParam, Ident, PathArguments, Type, TypePath, Variant, parse_quote,
    spanned::Spanned,
};

mod diagnostic;

use diagnostic::{
    DiagnosticResult::{self, Ok},
    DiagnosticStream,
};

#[proc_macro_derive(Try)]
/// Derives [try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html)
///
/// ## Requires:
///   - `RUSTC_BOOTSTRAP = 1` (or nightly)
///   - `#![feature(never_type)]`
///   - `#![feature(try_trait_v2)]`
///
/// ## Current Limitations on the annotated type:
///   - must be an `enum`
///   - must have _at least one_ generic type
///   - the output type must be the _first_ generic type
///   - the output variant (does not short-circuit) must be the _first_ variant
///     and **only** store _the output type_
pub fn try_trait_v2_derive(input: TokenStream1) -> TokenStream1 {
    impl_derive(input.into()).into()
}

/// A destructured Enum with validated invariants
struct TryEnum<'ast> {
    name: &'ast Ident,
    enum_data: &'ast DataEnum,
    output_variant: &'ast Ident,
    output_type: &'ast Type,
    output_type_name: &'ast Ident,
    residual_type: Type,
}

type BranchArm = Arm;
type ResidualArm = Arm;

impl<'ast> TryEnum<'ast> {
    fn try_parse(ast: &'ast DeriveInput) -> DiagnosticResult<Self> {
        // Fail fast
        let enum_data: &DataEnum = match &ast.data {
            Data::Enum(enum_data) => Ok(enum_data),
            Data::Struct(struct_data) => {
                DiagnosticResult::error("Try can only be derived for an enum")
                    .add_help(struct_data.struct_token.span(), "not an enum")
            }
            Data::Union(union_data) => {
                DiagnosticResult::error("Try can only be derived for an enum")
                    .add_help(union_data.union_token.span(), "not an enum")
            }
        }?;

        let name: &Ident = &ast.ident;

        // TODO: Check that multiline enum defs show whole def in help
        let (output_variant, output_type, output_type_name): (&Ident, &Type, &Ident) = {
            let first_generic_type: &Ident = match ast.generics.type_params().next() {
                Some(output_ty) => Ok(&output_ty.ident),
                None => DiagnosticResult::error("Try requires a generic type for `Output`")
                    .add_help(name.span(), "Add <T> after this..."),
            }?;
            let output_variant = enum_data.variants.first().ok_or(
                DiagnosticResult::error("Try cannot be derived for a zero-field enum").add_help(
                    enum_data.brace_token.span.span(),
                    "add at least two variants here...",
                ),
            )?;
            let output_type = match &output_variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let output_type = &fields
                        .unnamed
                        .first()
                        .expect("at least one unnamed field")
                        .ty;
                    let is_first_generic_type = |tp: &TypePath| match tp.path.get_ident() {
                        Some(var_ty) if var_ty == first_generic_type => Ok(()),
                        Some(var_ty) => DiagnosticResult::error(
                            "Try requires the first generic type to match the `Output` type",
                        )
                        .add_help(first_generic_type.span(), "Output type defined here")
                        .add_help(
                            var_ty.span(),
                            format_args!("change this to {first_generic_type}"),
                        ),
                        //TODO: handle borrowed. Combine NotPathOrRef & NotSimpleIdent.
                        None => DiagnosticResult::error("Try requires a generic type for `Output`")
                            .add_help(first_generic_type.span(), "Output type defined here")
                            .add_help(
                                fields
                                    .unnamed
                                    .first()
                                    .expect("at least one unnamed field")
                                    .span(),
                                format_args!("change this to {first_generic_type}"),
                            ),
                    };
                    match output_type {
                        Type::Path(tp) => is_first_generic_type(tp)?,
                        Type::Reference(tr) => match tr.elem.as_ref() {
                            Type::Path(tp) => is_first_generic_type(tp)?,
                            _ => todo!("{:?}", tr.elem.as_ref()),
                        },
                        // TODO: #18 Finalise and verify error messages (mainly spans) with borrows
                        _ => DiagnosticResult::error("Try requires a generic type for `Output`")
                            .add_help(first_generic_type.span(), "Output type defined here")
                            .add_help(
                                fields
                                    .unnamed
                                    .first()
                                    .expect("at least one unnamed field")
                                    .span(),
                                format_args!("change this to {first_generic_type}"),
                            )?,
                    };
                    output_type
                }
                Fields::Unnamed(fields) => {
                    let base_error: DiagnosticResult<&FieldsUnnamed> =
                        DiagnosticResult::error("Try requires a single generic type for `Output`")
                            .add_help(first_generic_type.span(), "Output type defined here");
                    let first_output_usage = &fields
                        .unnamed
                        .iter()
                        .find(|field| match &field.ty {
                            Type::Path(f) => {
                                f.path.get_ident().is_some_and(|t| t == first_generic_type)
                            }
                            Type::Reference(r) if let Type::Path(f) = r.elem.as_ref() => {
                                f.path.get_ident().is_some_and(|t| t == first_generic_type)
                            }
                            _ => todo!("other types 142"),
                        })
                        .ok_or_else(|| {
                            DiagnosticResult::error(
                                "Try requires a single generic type for `Output`",
                            )
                            .add_help(first_generic_type.span(), "Output type defined here")
                            .add_help(
                                fields.span(),
                                format_args!("change this to ({first_generic_type})"),
                            )
                        })?
                        .ty;
                    match first_output_usage {
                        Type::Path(_) => base_error.add_help(
                            fields.span(),
                            format_args!("change this to ({first_generic_type})"),
                        )?,
                        Type::Reference(r) => base_error.add_help(
                            fields.span(),
                            format_args!(
                                "change this to (&{} {first_generic_type})",
                                r.lifetime.as_ref().expect("generic ref must have lifetime")
                            ),
                        )?,
                        _ => todo!("too many fields AND THEN wrong type"),
                    };
                    unreachable!("all paths above diverge via ?")
                }
                Fields::Unit => DiagnosticResult::error("Try requires a generic type for `Output`")
                    .add_help(
                        output_variant.span(),
                        format_args!("add ({first_generic_type}) after this..."),
                    )?,
                Fields::Named(fields) => DiagnosticResult::error(
                    "Try requires an unnamed field for the `Output` variant",
                )
                .add_help(
                    fields.span(),
                    format_args!("change this to ({first_generic_type})"),
                )?,
            };
            Ok((&output_variant.ident, output_type, first_generic_type))
        }?;

        // Must be done late, after validating suitable generics
        let residual_type: Type = Self::generate_residual(ast);

        Ok(Self {
            name,
            enum_data,
            output_variant,
            output_type,
            output_type_name,
            residual_type,
        })
    }

    /// Generate the residual type with appropriate arguments (! + remaining generics).
    ///
    /// Infallible as we already guarantee we are processing an enum with at least one generic type.
    fn generate_residual(ast: &DeriveInput) -> Type {
        // Separate function to allow direct tests
        let name = &ast.ident;
        let (_, tygenerics, _) = ast.generics.split_for_impl();
        let mut residual_type: Type = parse_quote! {#name #tygenerics}; // e.g. `Foo<T,E,U>`
        let path_args: &mut AngleBracketedGenericArguments = {
            let Type::Path(ref mut residual_type) = residual_type else {
                unreachable!("enum name must be Type::Path")
            };
            match residual_type
                .path
                .segments
                .first_mut()
                .expect("valid enum definition has exactly one segment")
                .arguments
            {
                PathArguments::AngleBracketed(ref mut args) => args,
                _ => unreachable!("TypeGenerics quotes to angle bracketed arguments"),
            }
        };
        //change FIRST generic type to `!`
        path_args
            .args
            .iter_mut()
            .find_map(|arg| {
                if let &mut GenericArgument::Type(ref mut typ) = arg {
                    *typ = parse_quote!(!);
                    Some(arg) //break out of find_map
                } else {
                    None
                }
            })
            .expect("must have at least one generic output type");
        residual_type
    }

    fn generate_arms(
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
}

fn impl_derive(input: TokenStream2) -> DiagnosticStream {
    let ast: DeriveInput = syn::parse2(input).expect("derive macro");

    #[allow(unused_variables)]
    let TryEnum {
        name,
        enum_data,
        output_variant,
        output_type,
        output_type_name,
        residual_type,
    } = TryEnum::try_parse(&ast)?;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let (branch_arms, residual_arms) = TryEnum::generate_arms(name, enum_data, output_type);

    let impl_try = quote! {
        impl #impl_generics std::ops::Try for #name #ty_generics #where_clause {
            type Output = #output_type;

            type Residual = #residual_type;

            #[inline]
            fn from_output(output: Self::Output) -> Self {
                Self::#output_variant(output)
            }

            #[inline]
            fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                match self {
                    #(#branch_arms)*
                }
            }
        }

        impl #impl_generics std::ops::FromResidual<#residual_type> for #name #ty_generics #where_clause {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #residual_type) -> Self {
                match residual {
                    #(#residual_arms)*
                }
            }
        }
    };
    DiagnosticResult::Ok(impl_try)
}

#[proc_macro_derive(Try_ConvertResult)]
/// Derives conversion from Result<T, E> where E: Into::into(Self) and back.
///
/// Simply `impl<T> From<SpecificError> for MyTryEnum<T>` then use `?` on a
/// `Result<_, SpecificError>` in any function which returns `MyTryEnum<_>`
///
/// For conversion to a `Result<_, ErrorType>` `impl From<MyTryEnum<!>> for ErrorType`
pub fn try_trait_v2_convert_result(input: TokenStream1) -> TokenStream1 {
    impl_convert_result(input.into()).into()
}

// TODO: #17 Add diagnostics to impl_convert_result
fn impl_convert_result(input: TokenStream2) -> TokenStream2 {
    let ast: DeriveInput = syn::parse2(input).expect("derive macro");

    #[allow(unused_variables)]
    let TryEnum {
        name,
        enum_data,
        output_variant,
        output_type,
        output_type_name,
        residual_type,
    } = TryEnum::try_parse(&ast).unwrap();

    let (_, ty_generics, where_clause) = &ast.generics.split_for_impl();
    let result_e = format_ident!("Derive_TryConvert_ResultE");
    let result_t = format_ident!("Derive_TryConvert_ResultT");

    let mut from_result_generics = ast.generics.clone();
    from_result_generics
        .params
        .push(parse_quote! {#result_e: Into<#name #ty_generics>});
    let (from_result_impl_generics, _, _) = from_result_generics.split_for_impl();

    let mut to_result_generics = ast.generics.clone();
    to_result_generics.params = to_result_generics
        .params
        .into_iter()
        .filter(|p| {
            if let GenericParam::Type(t) = p {
                &t.ident != output_type_name
            } else {
                true
            }
        })
        .chain([
            parse_quote! {#result_t},
            parse_quote! {#result_e: From<#residual_type>},
        ])
        .collect();
    let (to_result_impl_generics, _, _) = to_result_generics.split_for_impl();

    quote! {
        impl #from_result_impl_generics std::ops::FromResidual<std::result::Result<std::convert::Infallible, #result_e>> for #name #ty_generics #where_clause
        {
            #[inline]
            #[track_caller]
            fn from_residual(residual: std::result::Result<std::convert::Infallible, #result_e>) -> Self {
                match residual {
                    Result::Err(e) => e.into(),
                }
            }
        }

        impl #to_result_impl_generics std::ops::FromResidual<#residual_type> for std::result::Result<#result_t, #result_e>
        {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #residual_type) -> Self {
                std::result::Result::Err(residual.into())
            }
        }
    }
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
        let residual = TryEnum::generate_residual(&original);
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
        let residual = TryEnum::generate_residual(&original);
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
        let residual = TryEnum::generate_residual(&original);
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
        let residual = TryEnum::generate_residual(&original);
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
        let residual = TryEnum::generate_residual(&original);
        let expected_residual: Type = parse_quote! {MyResult<'t, 'e, !, E>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn derive() {
        let original: TokenStream2 = quote! {
            #[derive(Try)]
            enum Exit<T: Termination> {
                Ok(T),
                TestsFailed,
                OtherError(String),
                NamedError{err: String, text: String},
            }
        };

        let derived_impl: TokenStream2 = quote! {
            impl<T: Termination> std::ops::Try for Exit<T> {
                type Output = T;

                type Residual = Exit<!>;

                #[inline]
                fn from_output(output: Self::Output) -> Self {
                    Self::Ok(output)
                }

                #[inline]
                fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                    match self {
                        Self::Ok(v0) => std::ops::ControlFlow::Continue(v0),
                        Self::TestsFailed => std::ops::ControlFlow::Break(Exit::TestsFailed),
                        Self::OtherError(v0) => std::ops::ControlFlow::Break(Exit::OtherError(v0)),
                        Self::NamedError{err, text} => std::ops::ControlFlow::Break(Exit::NamedError{err, text}),
                    }
                }
            }

            impl<T: Termination> std::ops::FromResidual<Exit<!> > for Exit<T> {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!>) -> Self {
                    match residual {
                        Exit::TestsFailed => Exit::TestsFailed,
                        Exit::OtherError(v0) => Exit::OtherError(v0),
                        Exit::NamedError{err, text} => Exit::NamedError{err, text},
                    }
                }
            }
        };
        assert_eq!(
            derived_impl.to_string(),
            impl_derive(original).unwrap().to_string()
        )
    }
    #[test]
    fn convert_result() {
        let original: TokenStream2 = quote! {
            #[derive(Try_ConvertResult)]
            enum Exit<T: Termination, E> {
                Ok(T),
                TestsFailed,
                OtherError(E),
            }
        };

        let expected_impl: TokenStream2 = quote! {
            impl<T: Termination, E, Derive_TryConvert_ResultE: Into< Exit<T, E> > > std::ops::FromResidual<std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>> for Exit<T, E>
            {
                #[inline]
                #[track_caller]
                fn from_residual(residual: std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>) -> Self {
                    match residual {
                        Result::Err(e) => e.into(),
                    }
                }
            }

            impl<E, Derive_TryConvert_ResultT, Derive_TryConvert_ResultE: From<Exit<!, E> > > std::ops::FromResidual<Exit<!, E> > for std::result::Result<Derive_TryConvert_ResultT, Derive_TryConvert_ResultE>
            {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!, E>) -> Self {
                    std::result::Result::Err(residual.into())
                }
            }
        };

        assert_eq!(
            expected_impl.to_string(),
            impl_convert_result(original).to_string()
        )
    }
}
