use proc_macro::TokenStream as TokenStream1;

mod derive;
use derive::*;

#[proc_macro_derive(TryTraitv2)]
pub fn try_trait_v2_derive(input: TokenStream1) -> TokenStream1 {
    impl_try_trait_v2(input.into()).into()
}
