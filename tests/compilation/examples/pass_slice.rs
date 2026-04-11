#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult};

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<'y, Y> {
    Yes(&'y [Y]),
    No,
}

fn main() {
    let a = [1,2,3,4,5,6];
    let s = &a[0..2];
    let d = *s;
    let a: [!; 6] = [!];
    let s = &a[0..2];
    let d = *s;
}
