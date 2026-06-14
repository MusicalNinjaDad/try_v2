#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum NotSimpleIdent<T> {
    Ok(proc_macro2::TokenStream),
    Err(T),
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum NotSimpleIdentBorrowed<'t, T> {
    Ok(&'t proc_macro2::TokenStream),
    Err(T),
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum NotPathOrRef<T> {
    Ok(!),
    Err(T),
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum RefToNotPath<'n, T> {
    Ok(&'n !),
    Err(T),
}

fn main() {}
