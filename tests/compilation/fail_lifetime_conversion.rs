#![feature(never_type)]
#![feature(try_trait_v2)]

//! Tests that conversion between custom enum and std::result::Result requires correct lifetime bounds.

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

#[derive(Debug)]
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

fn _unrestricted_t<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> Result<&'t i32, Failure<'e>>
where
    'input: 'e,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        _ => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_e<'input, 't, 'e>(
    okval: &'input i32,
    errval: &'input i32,
) -> Result<&'t i32, Failure<'e>>
where
    'input: 't,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        _ => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_t_from_result<'input, 't, 'e>(
    errmond: Result<&'input i32, &'input i32>,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'input: 'e,
{
    let val = errmond?;
    BorrowedResult::Ok(val)
}

fn _unrestricted_e_from_result<'input, 't, 'e>(
    errmond: Result<&'input i32, &'input i32>,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'input: 't,
{
    let val = errmond?;
    BorrowedResult::Ok(val)
}

fn main() {}
