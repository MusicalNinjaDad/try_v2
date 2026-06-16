#![allow(unused)]
#![allow(clippy::disallowed_names)]

use std::task::{Poll, ready};

fn wobble() -> Poll<Option<Result<(), ()>>> {
    let foo: Poll<Option<Result<(), ()>>> = Poll::Ready(Some(Ok(())));
    let bar = ready!(foo?);
    todo!()
}

fn main() {}
