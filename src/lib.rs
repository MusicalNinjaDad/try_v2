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
//!   - must have _one_ generic type
//!   - the _first and only_ generic type must be the `Output` type (produced when not short circuiting)
//!   - the output variant (does not short-circuit) must be the _first_ variant and store the output
//!     type as the _only unnamed_ field
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

use std::fmt::Display;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
    Arm, Data, DataEnum, DeriveInput, Fields, GenericParam, Ident, parse_quote, spanned::Spanned,
};

use crate::DiagnosticResult::Ok;

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

fn impl_derive(input: TokenStream2) -> DiagnosticStream {
    let ast: DeriveInput = syn::parse2(input).expect("derive macro");

    let name: &Ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let enum_data: &DataEnum = match &ast.data {
        Data::Enum(enum_data) => enum_data,
        Data::Struct(struct_data) => {
            return DiagnosticResult::error("Try can only be derived for an enum")
                .add_help(struct_data.struct_token.span(), "not an enum");
        }
        Data::Union(union_data) => {
            return DiagnosticResult::error("Try can only be derived for an enum")
                .add_help(union_data.union_token.span(), "not an enum");
        }
    };

    let output_ty: &Ident = match ast.generics.type_params().next() {
        Some(output_ty) => &output_ty.ident,
        None => {
            return DiagnosticResult::error("Try requires a generic type for `Output`")
                .add_help(name.span(), "Add <T> after this...");
        }
    };

    let output_variant: &Ident = parse_output_variant(enum_data, output_ty)?;

    let (branch, residual) = arms(name, enum_data);

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
                    #(#branch)*
                }
            }
        }

        impl #impl_generics std::ops::FromResidual<#name<!>> for #name #ty_generics #where_clause {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #name<!>) -> Self {
                match residual {
                    #(#residual)*
                }
            }
        }
    };
    DiagnosticResult::Ok(impl_try)
}

type BranchArms = Vec<Arm>;
type ResidualArms = Vec<Option<Arm>>;

fn arms(enum_name: &Ident, enumdata: &DataEnum) -> (BranchArms, ResidualArms) {
    enumdata.variants.iter().enumerate().map(
        |(i, variant)| {
            let var_name: &Ident = &variant.ident;
            let is_output_variant = i == 0;
            let (branch_arm, residual_arm);
            match(is_output_variant, &variant.fields) {
                (true, _) => {
                    // Output variant always has a single field
                    branch_arm = parse_quote! {
                        Self::#var_name(v0) => std::ops::ControlFlow::Continue(v0),
                    };
                    residual_arm = None;
                },
                (false, Fields::Unnamed(_)) => {
                    let vars: Vec<Ident> = (0..variant.fields.len())
                        .map(|n| format_ident!("v{n}"))
                        .collect();
                    branch_arm = parse_quote! {
                        Self::#var_name(#(#vars),*) => std::ops::ControlFlow::Break(#enum_name::#var_name(#(#vars),*)),
                    };
                    residual_arm = Some(parse_quote! {
                        #enum_name::#var_name(#(#vars),*) => #enum_name::#var_name(#(#vars),*),
                    });
                },
                (false, Fields::Unit) => {
                    branch_arm = parse_quote! {
                        Self::#var_name => std::ops::ControlFlow::Break(#enum_name::#var_name),
                    };
                    residual_arm = Some(parse_quote! {
                        #enum_name::#var_name => #enum_name::#var_name,
                    });
                },
                (false, Fields::Named(_)) => todo!("Error for or handle named fields")
            };
            (branch_arm, residual_arm)
        }
    ).unzip()
}

fn parse_output_variant<'ast>(
    enum_data: &'ast DataEnum,
    output_ty: &'ast Ident,
) -> DiagnosticResult<&'ast Ident> {
    let output_variant = enum_data.variants.first().ok_or(
        DiagnosticResult::error("Try cannot be derived for a zero-field enum").add_help(
            enum_data.brace_token.span.span(),
            "add at least two variants here...",
        ),
    )?;
    let fields = match &output_variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(fields),
        Fields::Unnamed(fields) => {
            DiagnosticResult::error("Try requires a single generic type for `Output`")
                .add_help(fields.span(), format_args!("change this to ({output_ty})"))
        }
        Fields::Unit => DiagnosticResult::error("Try requires a generic type for `Output`")
            .add_help(
                output_variant.span(),
                format_args!("add ({output_ty}) after this..."),
            ),
        Fields::Named(fields) => {
            DiagnosticResult::error("Try requires an unnamed field for the `Output` variant")
                .add_help(fields.span(), format_args!("change this to ({output_ty})"))
        }
    }?;
    let type_path = match &fields
        .unnamed
        .first()
        .expect("at least one unnamed field")
        .ty
    {
        syn::Type::Path(tp) => Ok(tp),
        _ => DiagnosticResult::error("Try requires a generic type for `Output`")
            .add_help(fields.span(), format_args!("change this to ({output_ty})")),
    }?;
    match type_path.path.get_ident() {
        Some(var_ty) if var_ty == output_ty => Ok(()),
        Some(var_ty) => DiagnosticResult::error(
            "Try requires the first generic type to match the `Output` type",
        )
        .add_help(output_ty.span(), "Output type defined here")
        .add_help(var_ty.span(), format_args!("change this to {output_ty}")),
        None => DiagnosticResult::error("Try requires a generic type for `Output`")
            .add_help(fields.span(), format_args!("change this to ({output_ty})")),
    }?;
    Ok(&output_variant.ident)
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
    let ast: DeriveInput = syn::parse2(input).unwrap();

    let name = &ast.ident;

    let (_, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let mut extended_generics = ast.generics.clone();
    let err_generic: GenericParam =
        syn::parse2(quote! {Derive_TryConvert_ResultE: Into<#name #ty_generics>}).unwrap();
    extended_generics.params.push(err_generic);

    let (impl_generics, _, _) = extended_generics.split_for_impl();

    quote! {
        impl #impl_generics std::ops::FromResidual<std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>> for #name #ty_generics #where_clause
        {
            #[inline]
            #[track_caller]
            fn from_residual(residual: std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>) -> Self {
                match residual {
                    Result::Err(e) => e.into(),
                }
            }
        }

        impl<T, E: From<#name<!>>> std::ops::FromResidual<#name<!>> for std::result::Result<T, E>
        {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #name<!>) -> Self {
                std::result::Result::Err(residual.into())
            }
        }
    }
}

type DiagnosticStream = DiagnosticResult<TokenStream2>;

#[derive(Debug)]
enum DiagnosticResult<T> {
    Ok(T),
    Err(Diagnostic),
}

impl<T> DiagnosticResult<T> {
    fn error<S: Display>(message: S) -> Self {
        Self::Err(Diagnostic {
            level: Level::Error,
            message: message.to_string(),
            spans: vec![Span::call_site()],
            children: vec![],
        })
    }
    fn add_help<S: Display>(mut self, span: Span, message: S) -> Self {
        let Self::Err(ref mut diagnostic) = self else {
            todo!()
        };
        diagnostic.children.push(Diagnostic {
            level: Level::Help,
            message: message.to_string(),
            spans: vec![span],
            children: vec![],
        });
        self
    }
    #[allow(unused)]
    fn unwrap(self) -> T {
        let Self::Ok(t) = self else {
            panic!("Called unwrap on a not-OK value")
        };
        t
    }
}

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

impl<T> std::ops::Try for DiagnosticResult<T> {
    type Output = T;

    type Residual = DiagnosticResult<!>;

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Ok(t) => std::ops::ControlFlow::Continue(t),
            Self::Err(d) => std::ops::ControlFlow::Break(DiagnosticResult::Err(d)),
        }
    }
}

impl<T> std::ops::FromResidual<DiagnosticResult<!>> for DiagnosticResult<T> {
    fn from_residual(residual: DiagnosticResult<!>) -> Self {
        match residual {
            DiagnosticResult::Err(residual) => DiagnosticResult::Err(residual),
        }
    }
}

impl<T> std::ops::FromResidual<Result<std::convert::Infallible, DiagnosticResult<T>>>
    for DiagnosticResult<T>
{
    fn from_residual(result: Result<std::convert::Infallible, DiagnosticResult<T>>) -> Self {
        match result {
            Err(e) => e,
        }
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

impl From<DiagnosticStream> for TokenStream1 {
    fn from(result: DiagnosticStream) -> Self {
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
    fn branch_with_field() {
        let input: DeriveInput = syn::parse2(quote!(
            enum foo<T> {
                Output(T),
                Variant1(i32),
            }
        ))
        .unwrap();
        let Data::Enum(enumdata) = &input.data else {
            panic!("Not an enum")
        };
        let (branch, residual) = arms(&input.ident, enumdata);
        let expected_branch: Vec<Arm> = parse_quote! {
            Self::Output(v0) => std::ops::ControlFlow::Continue(v0),
            Self::Variant1(v0) => std::ops::ControlFlow::Break(foo::Variant1(v0)),
        };
        assert_eq!(
            quote!(#(#expected_branch)*).to_string(),
            quote!(#(#branch)*).to_string()
        );
        let expected_residual: Arm = parse_quote! {
            foo::Variant1(v0) => foo::Variant1(v0),
        };
        assert_eq!(
            quote!(#expected_residual).to_string(),
            quote!(#(#residual)*).to_string()
        );
    }

    #[test]
    fn branch_with_fields() {
        let input: DeriveInput = syn::parse2(quote!(
            enum foo<T> {
                Output(T),
                Variant1(i32, i32, String),
            }
        ))
        .unwrap();
        let Data::Enum(enumdata) = &input.data else {
            panic!("Not an enum")
        };
        let (branch, residual) = arms(&input.ident, enumdata);
        let expected_branch: Vec<Arm> = parse_quote! {
            Self::Output(v0) => std::ops::ControlFlow::Continue(v0),
            Self::Variant1(v0,v1,v2) => std::ops::ControlFlow::Break(foo::Variant1(v0,v1,v2)),
        };
        assert_eq!(
            quote!(#(#expected_branch)*).to_string(),
            quote!(#(#branch)*).to_string()
        );
        let expected_residual: Arm = parse_quote! {
            foo::Variant1(v0,v1,v2) => foo::Variant1(v0,v1,v2),
        };
        assert_eq!(
            quote!(#expected_residual).to_string(),
            quote!(#(#residual)*).to_string()
        );
    }

    #[test]
    fn all_arm_types() {
        let input: DeriveInput = syn::parse2(quote!(
            enum foo<T> {
                Output(T),
                Residual1(i32),
                Residual2,
                Residual3(i32, i32, String),
            }
        ))
        .unwrap();
        let Data::Enum(enumdata) = &input.data else {
            panic!("Not an enum")
        };
        let (branch, residual) = arms(&input.ident, enumdata);
        let expected_branch: Vec<Arm> = parse_quote! {
            Self::Output(v0) => std::ops::ControlFlow::Continue(v0),
            Self::Residual1(v0) => std::ops::ControlFlow::Break(foo::Residual1(v0)),
            Self::Residual2 => std::ops::ControlFlow::Break(foo::Residual2),
            Self::Residual3(v0,v1,v2) => std::ops::ControlFlow::Break(foo::Residual3(v0,v1,v2)),
        };
        assert_eq!(
            quote!(#(#expected_branch)*).to_string(),
            quote!(#(#branch)*).to_string()
        );
        let expected_residual: Vec<Arm> = parse_quote! {
            foo::Residual1(v0) => foo::Residual1(v0),
            foo::Residual2 => foo::Residual2,
            foo::Residual3(v0,v1,v2) => foo::Residual3(v0,v1,v2),
        };
        assert_eq!(
            quote!(#(#expected_residual)*).to_string(),
            quote!(#(#residual)*).to_string()
        );
    }

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
                        Self::Ok(v0) => std::ops::ControlFlow::Continue(v0),
                        Self::TestsFailed => std::ops::ControlFlow::Break(Exit::TestsFailed),
                        Self::OtherError(v0) => std::ops::ControlFlow::Break(Exit::OtherError(v0)),
                    }
                }
            }

            impl<T: Termination> std::ops::FromResidual<Exit<!>> for Exit<T> {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!>) -> Self {
                    match residual {
                        Exit::TestsFailed => Exit::TestsFailed,
                        Exit::OtherError(v0) => Exit::OtherError(v0),
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

        let expected_impl: TokenStream2 = quote! {
            impl<T: Termination, Derive_TryConvert_ResultE: Into< Exit<T> > > std::ops::FromResidual<std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>> for Exit<T>
            {
                #[inline]
                #[track_caller]
                fn from_residual(residual: std::result::Result<std::convert::Infallible, Derive_TryConvert_ResultE>) -> Self {
                    match residual {
                        Result::Err(e) => e.into(),
                    }
                }
            }

            impl<T, E: From<Exit<!>>> std::ops::FromResidual<Exit<!>> for std::result::Result<T, E>
            {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!>) -> Self {
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
