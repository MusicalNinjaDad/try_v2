use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, GenericParam};

pub(crate) fn impl_try_trait_v2(input: TokenStream2) -> TokenStream2 {
    let ast = syn::parse2::<DeriveInput>(input).unwrap();

    let (impl_generics, ty_generics, where_clause) = &ast.generics.split_for_impl();

    let output_ty = ast.generics.params.first().unwrap();
    let GenericParam::Type(output_ty) = output_ty else {
        todo!()
    };
    let output_ty = &output_ty.ident;

    let Data::Enum(enum_data) = ast.data else {
        todo!()
    };
    let output_variant = &enum_data.variants[0].ident; //TODO: validate field type

    let residual_variants_unit: Vec<_> = enum_data
        .variants
        .iter()
        .skip(1)
        .filter(|variant| variant.fields.is_empty())
        .cloned()
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
                    #(Self::#residual_variants_unit => std::ops::ControlFlow::Break(#name::#residual_variants_unit)),*
                    #(Self::#residual_variants_with_fields(v) => std::ops::ControlFlow::Break(#name::#residual_variants_with_fields(v))),*
                }
            }
        }

        impl #impl_generics std::ops::FromResidual<#name<!>> for #name #ty_generics #where_clause {
            #[inline]
            #[track_caller]
            fn from_residual(residual: #name<!>) -> Self {
                match residual {
                    #(#name::#residual_variants_unit => #name::#residual_variants_unit),*
                    #(#name::#residual_variants_with_fields(v) => #name::#residual_variants_with_fields(v)),*
                }
            }
        }
    };
    impl_try
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
                        Self::TestsFailed => std::ops::ControlFlow::Break(Exit::TestsFailed)
                        Self::OtherError(v) => std::ops::ControlFlow::Break(Exit::OtherError(v))
                    }
                }
            }

            impl<T: Termination> std::ops::FromResidual<Exit<!>> for Exit<T> {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<!>) -> Self {
                    match residual {
                        Exit::TestsFailed => Exit::TestsFailed
                        Exit::OtherError(v) => Exit::OtherError(v)
                    }
                }
            }
        };
        assert_eq!(
            derived_impl.to_string(),
            impl_try_trait_v2(original).to_string()
        )
    }
}
