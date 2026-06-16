// TODO: #145 Using Output type in a Residual variant should fail with a better error message
// error[E0308]: mismatched types
//   --> crates/try_v2_derive/examples/wip_ShortCircuitT.rs:10:17
//    |
// 10 |   #[derive(Debug, Try, Try_ConvertResult)]
//    |                   ^^^ expected `!`, found type parameter `T`
// 11 |   #[must_use]
// 12 |   enum Exit<T, E> {
//    |        -    - found this type parameter
//    |  ______|
//    | |
// 13 | |     Ok(T),
// 14 | |     TestsFailed,
// 15 | |     OtherError(T, E),
//    | |______________- arguments to this enum variant are incorrect
//    |
//    = note:        expected type `!`
//            found type parameter `T`
// help: the type constructed contains `T` due to the type of the argument passed
//   --> crates/try_v2_derive/examples/wip_ShortCircuitT.rs:10:17
//    |
// 10 | #[derive(Debug, Try, Try_ConvertResult)]
//    |                 ^^^ this argument influences the type of `OtherError`
// note: tuple variant defined here
//   --> crates/try_v2_derive/examples/wip_ShortCircuitT.rs:15:5
//    |
// 15 |     OtherError(T, E),
//    |     ^^^^^^^^^^
//    = note: this error originates in the derive macro `Try` (in Nightly builds, run with -Z macro-backtrace for more info)



#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2_derive::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
#[must_use]
enum Exit<T, E> {
    Ok(T),
    TestsFailed,
    OtherError(T, E),
}

fn main() {}

