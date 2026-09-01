#![allow(stable_features)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

//! Tests to ensure that lifetimes are correctly passed through and live as long as expected.

use try_v2_derive::Try;

// Basic result with T & E borrowed
#[must_use]
#[derive(Debug, Try)]
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

fn _unrestricted_t<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
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

fn _unrestricted_e<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'input: 't,
{
    use BorrowedResult::Ok;

    let rtn = match errval {
        ..=4 => pass(okval)?,
        5 => fail(errval)?,
        6.. => fail_directly(errval)?,
    };
    Ok(rtn)
}

fn main() {}
