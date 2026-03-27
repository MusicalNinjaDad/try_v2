#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]

//! Tests to ensure that lifetimes are correctly passed through and live as long as expected.

use std::assert_matches::assert_matches;

use try_v2::{Try, Try_ConvertResult};

// Basic result with T & E borrowed
#[derive(Debug, Try, Try_ConvertResult)]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

fn fail<'t, 'e, T, E>(err: &'e E) -> BorrowedResult<'t, 'e, T, E> {
    let r = BorrowedResult::Err(err)?;
    BorrowedResult::Ok(r)
}

fn pass<'t, 'e, T, E>(val: &'t T) -> BorrowedResult<'t, 'e, T, E> {
    BorrowedResult::Ok(val)
}

fn validate_err_lifetime<'t, 'e>(
    okval: &'t i32,
    errval: &'e i32,
) -> BorrowedResult<'t, 'e, i32, i32> {
    use BorrowedResult::Ok;

    let rtn = match errval {
        ..=4 => pass(okval)?,
        _ => fail(errval)?,
    };
    Ok(rtn)
}

fn restricted_lifetimes<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'input: 't,
    'input: 'e,
{
    use BorrowedResult::Ok;

    let rtn = match errval + okval {
        ..=4 => pass(okval)?,
        _ => fail(errval)?,
    };
    Ok(rtn)
}

fn main() {
    assert_matches!(validate_err_lifetime(&0, &5), BorrowedResult::Err(&5));
    assert_matches!(restricted_lifetimes(&0, &5), BorrowedResult::Err(&6));
}
