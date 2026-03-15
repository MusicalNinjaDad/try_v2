//! Provides a derive macro for `Try`
//! ([try_trait_v2](https://rust-lang.github.io/rfcs/3058-try-trait-v2.html))
//!
//! Also enables auto-conversion from `Result<T, E> where E: Into::into(Self)`
//!
//! ## Requires:
//!   - `RUSTC_BOOTSTRAP = 1` (or nightly)
//!   - `#![feature(never_type)]`
//!   - `#![feature(try_trait_v2)]`
//!
//! ## Current Limitations on the annotated type:
//!   - must be an `enum`
//!   - must have _one_ generic type
//!   - the _first and only_ generic type must be the `Output` type (produced when not short circuiting)
//!   - the output variant (does not short-circuit) must be the _first_ variant
//!   - other (short-circuiting) variants can have _at most one unnamed field_
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
use quote::quote;
use syn::{Data, DeriveInput, GenericParam};

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
///   - must have _one_ generic type
///   - the _first & only_ generic type must be the `Output` type (produced when not short circuiting)
///   - the output variant (does not short-circuit) must be the _first_ variant
///   - other (short-circuiting) variants can have _at most one unnamed field_
pub fn try_trait_v2_derive(input: TokenStream1) -> TokenStream1 {
    impl_try_trait_v2(input.into()).into()
}

fn impl_try_trait_v2(input: TokenStream2) -> TokenStream2 {
    let ast: DeriveInput = syn::parse2(input).unwrap();

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let output_ty = match ast.generics.params.first().unwrap() {
        GenericParam::Type(output_ty) => &output_ty.ident,
        _ => todo!(),
    };

    let Data::Enum(enum_data) = ast.data else {
        todo!()
    };

    let output_variant = &enum_data.variants[0].ident; //TODO: validate field type

    let residual_variants_unit: Vec<_> = enum_data
        .variants
        .iter()
        .skip(1)
        .filter(|variant| variant.fields.is_empty())
        .map(|variant| variant.ident.clone())
        .collect();

    let residual_variants_with_fields: Vec<_> = enum_data
        .variants
        .iter()
        .skip(1)
        .filter(|variant| !variant.fields.is_empty())
        .map(|variant| variant.ident.clone())
        .collect(); //TODO: multiple fields

    let name = &ast.ident;

    let impl_try = quote! {
        impl #impl_generics std::ops::Try for #name #ty_generics #where_clause {
            type Output = #output_ty;

            type Residual = #name<!>;

            #[inline]
            fn from_output(output: Self::Output) -> Self {
                Self::#output_variant(output)
            }

            #[inline]
            fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Self::#output_variant(v) => std::ops::ControlFlow::Continue(v),
                    #(Self::#residual_variants_unit => std::ops::ControlFlow::Break(#name::#residual_variants_unit),)*
                    #(Self::#residual_variants_with_fields(v) => std::ops::ControlFlow::Break(#name::#residual_variants_with_fields(v)),)*
                }
            }
        }

        impl #impl_generics std::ops::FromResidual<#name<!>> for #name #ty_generics #where_clause {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #name<!>) -> Self {
                match residual {
                    #(#name::#residual_variants_unit => #name::#residual_variants_unit,)*
                    #(#name::#residual_variants_with_fields(v) => #name::#residual_variants_with_fields(v),)*
                }
            }
        }
    };
    impl_try
}

#[proc_macro_derive(Try_ConvertResult)]
/// Derives conversion from Result<T, E> where E: Into::into(Self)
/// 
/// Simply `impl<T> From<SpecificError> for MyTryEnum<T>` then use `?` on a
/// `Result<_, SpecificError>` in any function which returns `MyTryEnum<_>`
pub fn try_trait_v2_result_derive(input: TokenStream1) -> TokenStream1 {
    impl_try_trait_v2_result(input.into()).into()
}

fn impl_try_trait_v2_result(input: TokenStream2) -> TokenStream2 {
    let ast: DeriveInput = syn::parse2(input).unwrap();

    let name = &ast.ident;

    let (_, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut extended_generics = ast.generics.clone();
    let err_generic: GenericParam = syn::parse2(quote! {E: Into<#name #ty_generics>}).unwrap();
    extended_generics.params.push(err_generic);

    let (impl_generics, _, _) = extended_generics.split_for_impl();

    quote! {
        impl #impl_generics std::ops::FromResidual<std::result::Result<std::convert::Infallible, E>> for #name #ty_generics #where_clause
        {
            #[inline]
            #[track_caller]
            fn from_residual(residual: std::result::Result<std::convert::Infallible, E>) -> Self {
                match residual {
                    Result::Err(e) => e.into(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive() {
        let original: TokenStream2 = quote! {
            #[derive(Try)]
            enum Exit<T: Termination> {
                Ok(T),
                TestsFailed,
                OtherError(String),
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
                        Self::Ok(v) => std::ops::ControlFlow::Continue(v),
                        Self::TestsFailed => std::ops::ControlFlow::Break(Exit::TestsFailed),
                        Self::OtherError(v) => std::ops::ControlFlow::Break(Exit::OtherError(v)),
                    }
                }
            }

            impl<T: Termination> std::ops::FromResidual<Exit<!>> for Exit<T> {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!>) -> Self {
                    match residual {
                        Exit::TestsFailed => Exit::TestsFailed,
                        Exit::OtherError(v) => Exit::OtherError(v),
                    }
                }
            }
        };
        assert_eq!(
            derived_impl.to_string(),
            impl_try_trait_v2(original).to_string()
        )
    }
    #[test]
    fn convert_result() {
        let original: TokenStream2 = quote! {
            #[derive(Try_ConvertResult)]
            enum Exit<T: Termination> {
                Ok(T),
                TestsFailed,
                OtherError(String),
            }
        };

        let derived_impl: TokenStream2 = quote! {
            impl<T: Termination, E: Into< Exit<T> > > std::ops::FromResidual<std::result::Result<std::convert::Infallible, E>> for Exit<T>
            {
                #[inline]
                #[track_caller]
                fn from_residual(residual: std::result::Result<std::convert::Infallible, E>) -> Self {
                    match residual {
                        Result::Err(e) => e.into(),
                    }
                }
            }
        };
        assert_eq!(
            derived_impl.to_string(),
            impl_try_trait_v2_result(original).to_string()
        )
    }
}
