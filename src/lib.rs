#![feature(import_trait_associated_functions)]
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
    AngleBracketedGenericArguments, Arm, Data, DataEnum, DeriveInput, Fields, GenericArgument,
    GenericParam, Ident, PathArguments, Type, TypePath, Variant, parse_quote, spanned::Spanned,
};

use std::ops::Not::not;

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
            let field = match &output_variant.fields {
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(fields),
                Fields::Unnamed(fields) => {
                    DiagnosticResult::error("Try requires a single generic type for `Output`")
                        .add_help(
                            fields.span(),
                            format_args!("change this to ({first_generic_type})"),
                        )
                }
                Fields::Unit => DiagnosticResult::error("Try requires a generic type for `Output`")
                    .add_help(
                        output_variant.span(),
                        format_args!("add ({first_generic_type}) after this..."),
                    ),
                Fields::Named(fields) => DiagnosticResult::error(
                    "Try requires an unnamed field for the `Output` variant",
                )
                .add_help(
                    fields.span(),
                    format_args!("change this to ({first_generic_type})"),
                ),
            }?;
            let output_type = &field
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
                None => DiagnosticResult::error("Try requires a generic type for `Output`")
                    .add_help(
                        field.span(),
                        format_args!("change this to ({first_generic_type})"),
                    ),
            };
            match output_type {
                Type::Path(tp) => is_first_generic_type(tp),
                Type::Reference(tr) => match tr.elem.as_ref() {
                    Type::Path(tp) => is_first_generic_type(tp),
                    _ => todo!("{:?}", tr.elem.as_ref()),
                },
                _ => DiagnosticResult::error("Try requires a generic type for `Output`").add_help(
                    field.span(),
                    format_args!("change this to ({first_generic_type})"),
                ),
            }?;
            Ok((&output_variant.ident, output_type, first_generic_type))
        }?;

        // Must be done late, after validating suitable generics
        let residual_type: Type = Self::generate_residual(ast, output_type, enum_data);

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
    fn generate_residual(ast: &DeriveInput, output_type: &Type, enum_data: &DataEnum) -> Type {
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
        path_args
            .args
            .iter_mut()
            // using find_map relies on invariant: first generic type is output type
            .find_map(|a| {
                let &mut GenericArgument::Type(ref mut t) = a else {
                    return None;
                };
                *t = parse_quote!(!);
                Some(a)
            })
            .expect("must have at least one generic output type");
        if let Type::Reference(output_type) = output_type {
            let output_lifetime = output_type
                .lifetime
                .as_ref()
                .expect("enum variants must use explicit lifetimes for stored refs");
            let uses_output_lifetime = |variant: &Variant| {
                variant.fields.iter().any(|field| {
                    if let Type::Reference(r) = &field.ty {
                        r.lifetime.as_ref() == Some(output_lifetime)
                    } else {
                        false
                    }
                })
            };
            let mut residual_variants = enum_data.variants.iter().skip(1);
            if !residual_variants.any(uses_output_lifetime) {
                let wanted_args = path_args.args.clone().into_iter().filter(|arg| {
                    not(matches!(arg, GenericArgument::Lifetime(lt) if lt == output_lifetime))
                });
                path_args.args = wanted_args.collect();
            }
        };
        residual_type
    }

    fn generate_arms(
        enum_name: &Ident,
        enum_data: &DataEnum,
    ) -> (Vec<BranchArm>, Vec<Option<ResidualArm>>) {
        let arms = |(i, variant): (usize, &Variant)| -> (BranchArm, Option<ResidualArm>) {
            let var_name: &Ident = &variant.ident;
            let is_output_variant = i == 0;
            match &variant.fields {
                _ if is_output_variant => {
                    // Output variant always has a single field
                    let branch_arm = parse_quote! {
                        Self::#var_name(v0) => std::ops::ControlFlow::Continue(v0),
                    };
                    (branch_arm, None)
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
    let (branch_arms, residual_arms) = TryEnum::generate_arms(name, enum_data);

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
        let Data::Enum(enum_data) = &original.data else {
            panic!()
        };
        let output_type: Type = parse_quote!(T);
        let residual = TryEnum::generate_residual(&original, &output_type, enum_data);
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
        let Data::Enum(enum_data) = &original.data else {
            panic!()
        };
        let output_type: Type = parse_quote!(T);
        let residual = TryEnum::generate_residual(&original, &output_type, enum_data);
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
        let Data::Enum(enum_data) = &original.data else {
            panic!()
        };
        let output_type: Type = parse_quote!(&'static T);
        let residual = TryEnum::generate_residual(&original, &output_type, enum_data);
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
        let Data::Enum(enum_data) = &original.data else {
            panic!()
        };
        let output_type: Type = parse_quote!(&'r T);
        let residual = TryEnum::generate_residual(&original, &output_type, enum_data);
        let expected_residual: Type = parse_quote! {MyResult<'r, !, E>};
        assert_eq!(expected_residual, residual);
    }

    #[test]
    fn mutliple_lifetimes_ref_residual() {
        let original: DeriveInput = parse_quote! {
            #[derive(Try)]
            enum MyResult<'t, 'e, T, E> {
                Ok(&'t T),
                Err(&'e E),
            }
        };
        let Data::Enum(enum_data) = &original.data else {
            panic!()
        };
        let output_type: Type = parse_quote!(&'t T);
        let residual = TryEnum::generate_residual(&original, &output_type, enum_data);
        let expected_residual: Type = parse_quote! {MyResult<'e, !, E>};
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
