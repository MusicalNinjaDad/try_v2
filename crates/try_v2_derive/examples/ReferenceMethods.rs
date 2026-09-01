#![allow(unused)]
#![cfg_attr(unstable_never_type, feature(never_type))]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::ops::{Deref, DerefMut, Try};

use try_v2_derive::Try;

#[derive(Debug, Try, PartialEq)]
#[methods(as_ref, as_mut, as_deref, as_deref_mut)]
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
    assert_eq!(
        EightBall::<&mut str, &mut i32>::Yes(&mut "yes".to_string()),
        EightBall::<String, i32>::Yes("yes".to_string()).as_deref_mut()
    );
    assert_eq!(
        EightBall::<&mut str, &mut i32>::RollAgain,
        EightBall::<String, i32>::RollAgain.as_deref_mut()
    );
    assert_eq!(
        EightBall::<&mut str, &mut i32>::No(&mut 5),
        EightBall::<String, i32>::No(5).as_deref_mut()
    );
}

fn main() {}
