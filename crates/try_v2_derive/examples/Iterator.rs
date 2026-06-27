#![allow(unused)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::Try;

use try_v2_derive::{IntoIterator, Try};

#[derive(Debug, Try, IntoIterator, PartialEq)]
#[methods]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

#[test]
fn into() {
    assert_eq!(Some(5), EightBall::<i32, i32>::Yes(5).into_iter().next());
    assert_eq!(None, EightBall::<i32, i32>::RollAgain.into_iter().next());
    assert_eq!(None, EightBall::<i32, i32>::No(5).into_iter().next());
    assert_eq!(Some(&5), EightBall::<i32, i32>::Yes(5).iter().next());
    let mut ball: EightBall<i32, i32> = EightBall::Yes(5);
    let n = ball.iter_mut().next().expect("was yes");
    assert_eq!(&mut 5, n);
    *n = 6;
    assert_eq!(EightBall::Yes(6), ball);
}

fn main() {}
