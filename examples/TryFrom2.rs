#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(associated_type_defaults)]

use try_v2::{Try, Try_ConvertResult};

/// Make TryFrom able to return arbitrary Try types
trait TryFrom2<T>: std::marker::Sized {
    /// Must keep, otherwise would be a breaking change
    type Error;
    /// The specific Try-type to return
    /// Defaults to Result, to make this a non-breaking change
    type Return: std::ops::Try = Result<Self, Self::Error>;

    fn try_from2(value: T) -> Self::Return;
}

#[derive(Try, Try_ConvertResult)]
#[must_use]
enum Eightball<Y> {
    Yes(Y),
    No,
}

struct Even(i32);

/// Uses new API to return a _custom_ TryType
impl TryFrom2<i32> for Even {
    /// Can always be set to `()` if irrelevant
    type Error = ();
    type Return = Eightball<Self>;

    fn try_from2(num: i32) -> Self::Return {
        if num % 2 == 0 {
            Eightball::Yes(Even(num))
        } else {
            Eightball::No
        }
    }
}

struct Even2(i32);

/// Uses _current hack_ of wrapping custom Try-type's Residual in a Result
impl TryFrom2<i32> for Even2 {
    type Error = Eightball<!>;

    fn try_from2(num: i32) -> Result<Even2, Eightball<!>> {
        if num % 2 == 0 {
            Result::Ok(Even2(num))
        } else {
            Result::Err(Eightball::No)
        }
    }
}

// Cannot instantiate a std::num::TryFromIntError
struct TryFromIntError;

/// Shows this is **non-breaking change**: this is identical (text) to std impl
impl TryFrom2<i8> for u8 {
    type Error = TryFromIntError;

    fn try_from2(u: i8) -> Result<Self, Self::Error> {
        if u >= 0 {
            Ok(u as Self)
        } else {
            Err(TryFromIntError)
        }
    }
}

/// Can call try_from in a function returning same try type, with different generics
fn even_string_own_try_type(num: i32) -> Eightball<String> {
    let n = Even::try_from2(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

/// Can call try_from in a function returning a _different_ try type (this goes via Result)
/// as long as a suitable FromResidual implementation exists
fn even_string_via_result(num: i32) -> Eightball<String> {
    let n = Even2::try_from2(num)?;
    let s = format!("{}", n.0);
    Eightball::Yes(s)
}

/// Current case not broken
fn unsigned(num: i8) -> Result<String, TryFromIntError> {
    let n = u8::try_from2(num)?;
    let s = format!("{}", n);
    Ok(s)
}

struct ThirdWord(String);

/// Could even return an Option
impl TryFrom2<&str> for ThirdWord {
    type Error = ();

    type Return = Option<Self>;

    fn try_from2(input: &str) -> Self::Return {
        input
            .split_whitespace()
            .nth(2)
            .map(|s| ThirdWord(s.to_string()))
    }
}

fn main() {
    assert!(matches!(even_string_own_try_type(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string_own_try_type(1), Eightball::No));

    assert!(matches!(even_string_via_result(2), Eightball::Yes(s) if s == "2"));
    assert!(matches!(even_string_via_result(1), Eightball::No));

    assert!(matches!(unsigned(5), Ok(s) if s == "5"));
    assert!(matches!(unsigned(-1), Err(TryFromIntError)));

    assert!(matches!(ThirdWord::try_from2("a lot of words"), Some(s) if s.0 == "of"));
    assert!(ThirdWord::try_from2("two words").is_none());
}
