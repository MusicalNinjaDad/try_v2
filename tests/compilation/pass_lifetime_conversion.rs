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
            BorrowedResult::Ok(&never) => match never {},
        }
    }
}

impl<'t, 'e, T> From<&'e i32> for BorrowedResult<'t, 'e, T, i32> {
    fn from(e: &'e i32) -> Self {
        BorrowedResult::Err(e)
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

fn result_to_borrowed_passthrough<'t, 'e>(
    errmond: Result<&'t i32, &'e i32>,
) -> BorrowedResult<'t, 'e, i32, i32> {
    let val = errmond?;
    BorrowedResult::Ok(val)
}

fn result_to_borrowed_restricted<'input, 't, 'e>(
    errmond: Result<&'input i32, &'input i32>,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'input: 't,
    'input: 'e,
{
    let val = errmond?;
    BorrowedResult::Ok(val)
}

fn main() {
    assert_matches!(validate_passthrough_lifetime(&0, &1), Result::Ok(&0));
    assert_matches!(validate_passthrough_lifetime(&0, &5), Result::Err(Failure(&5)));
    assert_matches!(validate_passthrough_lifetime(&0, &7), Result::Err(Failure(&7)));
    assert_matches!(restricted_lifetimes(&0, &1), Result::Ok(&0));
    assert_matches!(restricted_lifetimes(&0, &5), Result::Err(Failure(&5)));
    assert_matches!(restricted_lifetimes(&0, &7), Result::Err(Failure(&7)));
}
