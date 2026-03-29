#![feature(never_type)]
#![feature(try_trait_v2)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
enum ZeroFieldEnum<T> {}

fn main() {}
