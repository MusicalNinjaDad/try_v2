#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

//! Tests to ensure that lifetimes are correctly passed through and live as long as expected.

use std::assert_matches::assert_matches;

use try_v2::{Try, Try_ConvertResult};

// Basic result with T & E borrowed
#[derive(Debug, Try, Try_ConvertResult)]
#[must_use]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

fn fail<'t, 'e, T, E>(err: &'e E) -> BorrowedResult<'t, 'e, T, E> {
    let r = BorrowedResult::Err(err)?;
    BorrowedResult::Ok(r)
}

fn fail_directly<'t, 'e, T, E>(err: &'e E) -> BorrowedResult<'t, 'e, T, E> {
    BorrowedResult::Err(err)
}

fn pass<'t, 'e, T, E>(val: &'t T) -> BorrowedResult<'t, 'e, T, E> {
    BorrowedResult::Ok(val)
}

fn validate_passthrough_lifetime<'t, 'e>(
    okval: &'t i32,
    errval: &'e i32,
) -> BorrowedResult<'t, 'e, i32, i32> {
    use BorrowedResult::Ok;

    let rtn = match errval {
        ..=4 => pass(okval)?,
        5 => fail(errval)?,
        6.. => fail_directly(errval)?,
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

    let rtn = match errval {
        ..=4 => pass(okval)?,
        5 => fail(errval)?,
        6.. => fail_directly(errval)?,
    };
    Ok(rtn)
}

fn main() {
    assert_matches!(
        validate_passthrough_lifetime(&0, &1),
        BorrowedResult::Ok(&0)
    );
    assert_matches!(
        validate_passthrough_lifetime(&0, &5),
        BorrowedResult::Err(&5)
    );
    assert_matches!(
        validate_passthrough_lifetime(&0, &7),
        BorrowedResult::Err(&7)
    );
    assert_matches!(restricted_lifetimes(&0, &1), BorrowedResult::Ok(&0));
    assert_matches!(restricted_lifetimes(&0, &5), BorrowedResult::Err(&5));
    assert_matches!(restricted_lifetimes(&0, &7), BorrowedResult::Err(&7));
}
