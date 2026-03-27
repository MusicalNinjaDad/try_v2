#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]

//! Tests conversion between custom enum and std::result::Result with lifetimes.

use std::assert_matches::assert_matches;
use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

#[derive(Debug, PartialEq, Eq)]
struct Failure<'e>(&'e i32);

impl<'a, 'e> From<BorrowedResult<'a, 'e, !, i32>> for Failure<'e> {
    fn from(res: BorrowedResult<'a, 'e, !, i32>) -> Self {
        match res {
            BorrowedResult::Err(e) => Failure(e),
            BorrowedResult::Ok(never) => match never {},
        }
    }
}

fn validate_passthrough_lifetime<'t, 'e>(
    okval: &'t i32,
    errval: &'e i32,
) -> Result<&'t i32, Failure<'e>> {
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn restricted_lifetimes<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> Result<&'t i32, Failure<'e>>
where
    'input: 't,
    'input: 'e,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn main() {
    assert_matches!(validate_passthrough_lifetime(&0, &1), Result::Ok(&0));
    assert_matches!(validate_passthrough_lifetime(&0, &5), Result::Err(Failure(&5)));
    assert_matches!(validate_passthrough_lifetime(&0, &7), Result::Err(Failure(&7)));
    assert_matches!(restricted_lifetimes(&0, &1), Result::Ok(&0));
    assert_matches!(restricted_lifetimes(&0, &5), Result::Err(Failure(&5)));
    assert_matches!(restricted_lifetimes(&0, &7), Result::Err(Failure(&7)));
}
