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

    let Data::Enum(enum_data) = ast.data else {todo!()};
    let output_variant = &enum_data.variants[0].ident; //TODO: validate field type
    
    let name = &ast.ident;
    
    let impl_try = quote! {
        impl #impl_generics Try for #name #ty_generics #where_clause {
            type Output = #output_ty;

            type Residual = #name<!>;

            #[inline]
            fn from_output(output: Self::Output) -> Self {
                Self::#output_variant(output)
            }

            #[inline]
            fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                match self {
                    Self::Ok(v) => ControlFlow::Continue(v),
                    Self::TestsFailed => ControlFlow::Break(Exit::TestsFailed),
                }
            }
        }   

        impl<T: Termination> FromResidual<Exit<Infallible>> for Exit<T> {
            #[inline]
            #[track_caller]
            fn from_residual(residual: Exit<Infallible>) -> Self {
                match residual {
                    Exit::TestsFailed => Exit::TestsFailed,
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
            #[derive(TryTraitv2)]
            enum Exit<T: Termination> {
                Ok(T),
                TestsFailed,
            }
        };
        let derived_impl: TokenStream2 = quote! {
            impl<T: Termination> Try for Exit<T> {
                type Output = T;

                type Residual = Exit<!>;

                #[inline]
                fn from_output(output: Self::Output) -> Self {
                    Self::Ok(output)
                }

                #[inline]
                fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
                    match self {
                        Self::Ok(v) => ControlFlow::Continue(v),
                        Self::TestsFailed => ControlFlow::Break(Exit::TestsFailed),
                    }
                }
            }

            impl<T: Termination> FromResidual<Exit<Infallible>> for Exit<T> {
                #[inline]
                #[track_caller]
                fn from_residual(residual: Exit<Infallible>) -> Self {
                    match residual {
                        Exit::TestsFailed => Exit::TestsFailed,
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
