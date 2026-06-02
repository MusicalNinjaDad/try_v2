#![feature(never_type)]
#![allow(dead_code)]

enum ValidatedBox<T> {
    ValidValue(Box<T>),
    InvalidValue,
}

fn main() {
    use ValidatedBox::{InvalidValue, ValidValue};

    let x: ValidatedBox<!> = InvalidValue;
    let mut y = 0;

    y += match x {
        InvalidValue => 1,
        ValidValue(_) => unreachable!("no way to construct a Box<!>"),
    };

    y += match x {
        InvalidValue => 1,
        ValidValue(b) => match *b {},
    };

    assert_eq!(y, 2);
}
