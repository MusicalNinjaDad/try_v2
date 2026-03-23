#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum OutputNotSimpleIdent<T> {
    Ok(proc_macro2::TokenStream),
    Err(T),
}

fn main() {}
