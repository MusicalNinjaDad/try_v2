#![allow(unused)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::Try;

use try_v2_derive::Try;

#[derive(Debug, Try, PartialEq)]
#[methods(as_ref, as_mut)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[test]
fn refs() {
    assert_eq!(
        EightBall::<&i32, &i32>::Yes(&5),
        EightBall::<i32, i32>::Yes(5).as_ref()
    );
    assert_eq!(
        EightBall::<&i32, &i32>::RollAgain,
        EightBall::<i32, i32>::RollAgain.as_ref()
    );
    assert_eq!(
        EightBall::<&i32, &i32>::No(&5),
        EightBall::<i32, i32>::No(5).as_ref()
    );
    assert_eq!(
        EightBall::<&mut i32, &mut i32>::Yes(&mut 5),
        EightBall::<i32, i32>::Yes(5).as_mut()
    );
    assert_eq!(
        EightBall::<&mut i32, &mut i32>::RollAgain,
        EightBall::<i32, i32>::RollAgain.as_mut()
    );
    assert_eq!(
        EightBall::<&mut i32, &mut i32>::No(&mut 5),
        EightBall::<i32, i32>::No(5).as_mut()
    );
}

fn main() {}
