#![allow(unused)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::{Deref, Try};

use try_v2_derive::Try;

#[derive(Debug, Try, PartialEq)]
#[methods(as_ref, as_mut)]
#[must_use]
enum EightBall<Y, N> {
    Yes(Y),
    RollAgain,
    No(N),
}

impl<Y: Deref,N> EightBall<Y,N> {
    fn as_deref(&self) -> EightBall<&<Y as Deref>::Target, &N> {
        match self {
            EightBall::Yes(y) => EightBall::Yes(y),
            EightBall::RollAgain => EightBall::RollAgain,
            EightBall::No(n) => EightBall::No(n),
        }
    }
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

#[test]
fn derefs() {
    assert_eq!(
        EightBall::<&str, &i32>::Yes("yes"),
        EightBall::<String, i32>::Yes("yes".to_string()).as_deref()
    );
    assert_eq!(
        EightBall::<&str, &i32>::RollAgain,
        EightBall::<String, i32>::RollAgain.as_deref()
    );
    assert_eq!(
        EightBall::<&str, &i32>::No(&5),
        EightBall::<String, i32>::No(5).as_deref()
    );
}

fn main() {}
