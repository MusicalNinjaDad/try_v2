#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::Try;

#[derive(Try)]
enum NoGenerics {
    Ok,
    TestsFailed,
    OtherError(String),
}

#[test]
fn foo () {
    todo!()
}