#![feature(never_type)]
#![feature(proc_macro_diagnostic)]
#![feature(try_trait_v2)]

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
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DataEnum, DeriveInput, Fields, GenericParam, Ident, spanned::Spanned};

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
///   - other (short-circuiting) variants can have _at most one unnamed field_
pub fn try_trait_v2_derive(input: TokenStream1) -> TokenStream1 {
    impl_derive(input.into()).into()
}

fn impl_derive(input: TokenStream2) -> DiagnosticResult {
    let ast: DeriveInput = syn::parse2(input).unwrap();

    let name: &Ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let enum_data: &DataEnum = match &ast.data {
        Data::Enum(enum_data) => enum_data,
        Data::Struct(struct_data) => {
            return DiagnosticResult::error(
                Span::call_site(),
                "Try can only be derived for an enum",
            )
            .add_help(struct_data.struct_token.span(), "not an enum");
        }
        Data::Union(union_data) => {
            return DiagnosticResult::error(
                Span::call_site(),
                "Try can only be derived for an enum",
            )
            .add_help(union_data.union_token.span(), "not an enum");
        }
    };

    let output_ty: &Ident = match ast.generics.type_params().next() {
        Some(output_ty) => &output_ty.ident,
        None => {
            return DiagnosticResult::error(
                Span::call_site(),
                "Try requires a generic type for `Output`",
            )
            .add_help(name.span(), "Add <T> after this...");
        }
    };

    let output_variant = enum_data.variants.first().unwrap();
    let Fields::Unnamed(fields) = &output_variant.fields else {
        todo!()
    };
    // TODO: check only one
    let syn::Type::Path(type_path) = &fields.unnamed.first().unwrap().ty else {
        todo!()
    };
    let var_ty = type_path.path.get_ident().unwrap();
    if var_ty != output_ty {
        return DiagnosticResult::error(
            Span::call_site(),
            "Try requires the first generic type to match the `Output` type",
        )
        .add_help(output_ty.span(), "Output type defined here")
        .add_help(var_ty.span(), format!("This should be <{output_ty}>"));
    }

    let output_variant: &Ident = &output_variant.ident;

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
    DiagnosticResult::Ok(impl_try)
}

#[proc_macro_derive(Try_ConvertResult)]
/// Derives conversion from Result<T, E> where E: Into::into(Self)
///
/// Simply `impl<T> From<SpecificError> for MyTryEnum<T>` then use `?` on a
/// `Result<_, SpecificError>` in any function which returns `MyTryEnum<_>`
pub fn try_trait_v2_convert_result(input: TokenStream1) -> TokenStream1 {
    impl_convert_result(input.into()).into()
}

fn impl_convert_result(input: TokenStream2) -> TokenStream2 {
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

#[derive(Debug)]
enum DiagnosticResult {
    Ok(TokenStream2),
    Err(Diagnostic),
}

impl DiagnosticResult {
    fn error<S: Into<String>>(span: Span, message: S) -> Self {
        Self::Err(Diagnostic {
            level: Level::Error,
            message: message.into(),
            spans: vec![span],
            children: vec![],
        })
    }
    fn add_help<S: Into<String>>(mut self, span: Span, message: S) -> Self {
        let Self::Err(ref mut diagnostic) = self else {
            todo!()
        };
        diagnostic.children.push(Diagnostic {
            level: Level::Help,
            message: message.into(),
            spans: vec![span],
            children: vec![],
        });
        self
    }
    #[allow(unused)]
    fn unwrap(self) -> TokenStream2 {
        let Self::Ok(t) = self else {
            panic!("Called unwrap on a not-OK value: {:?}", self)
        };
        t
    }
}

struct DiagnosticResidual(Diagnostic);

#[derive(Debug, Clone)]
struct Diagnostic {
    level: Level,
    message: String,
    spans: Vec<Span>,
    children: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Level {
    Error,
    #[expect(unused)]
    Warning,
    #[expect(unused)]
    Note,
    Help,
}

impl From<Level> for proc_macro::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => Self::Error,
            Level::Help => Self::Help,
            Level::Note => Self::Note,
            Level::Warning => Self::Warning,
        }
    }
}

impl std::ops::Try for DiagnosticResult {
    type Output = TokenStream2;

    type Residual = DiagnosticResidual;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(t) => std::ops::ControlFlow::Continue(t),
            Self::Err(d) => std::ops::ControlFlow::Break(DiagnosticResidual(d)),
        }
    }
}

impl std::ops::FromResidual<DiagnosticResidual> for DiagnosticResult {
    fn from_residual(residual: DiagnosticResidual) -> Self {
        DiagnosticResult::Err(residual.0)
    }
}

impl Diagnostic {
    fn add_as_child(self, parent: proc_macro::Diagnostic) -> proc_macro::Diagnostic {
        let msg = self.message.clone();
        match self.level {
            Level::Error => parent.span_error(self.as_spans(), msg),
            Level::Warning => parent.span_warning(self.as_spans(), msg),
            Level::Note => parent.span_note(self.as_spans(), msg),
            Level::Help => parent.span_help(self.as_spans(), msg),
        }
    }
}

impl Diagnostic {
    fn as_spans(&self) -> Vec<proc_macro::Span> {
        self.spans.iter().map(|span| span.unwrap()).collect()
    }
}

impl From<DiagnosticResult> for TokenStream1 {
    fn from(result: DiagnosticResult) -> Self {
        match result {
            DiagnosticResult::Ok(t) => t.into(),
            DiagnosticResult::Err(diagnostic) => {
                // MSV: unwrap requires rustc 1.29+ *without* semver exempt features
                let spans = diagnostic.as_spans();
                let mut pm_diagnostic = proc_macro::Diagnostic::spanned(
                    spans,
                    diagnostic.level.into(),
                    diagnostic.message,
                );
                for child in diagnostic.children {
                    pm_diagnostic = child.add_as_child(pm_diagnostic);
                }
                pm_diagnostic.emit();
                TokenStream1::new()
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
            impl_derive(original).unwrap().to_string()
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
            impl_convert_result(original).to_string()
        )
    }
}
