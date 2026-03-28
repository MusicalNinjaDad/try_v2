#![feature(never_type)]
#![feature(try_trait_v2)]

//! Tests that conversion between custom enum and std::result::Result requires correct lifetime bounds.

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

impl<'pass, 'fail, 't, 'e, T, E> BorrowedResult<'t, 'e, T, E>
where
    'pass: 't,
    'fail: 'e,
{
    fn fail(err: &'fail E) -> Self {
        let r = Self::Err(err)?;
        Self::Ok(r)
    }

    fn fail_directly(err: &'fail E) -> Self {
        Self::Err(err)
    }

    fn pass(val: &'pass T) -> Self {
        Self::Ok(val)
    }
}

type StdResult<'o, 'f> = std::result::Result<&'o i32, Failure<'f>>;

#[derive(Debug, PartialEq, Eq)]
struct Failure<'f>(&'f i32);

fn fail<'fail, 'o, 'f>(err: Failure<'fail>) -> StdResult<'o, 'f>
where
    'fail: 'f,
{
    let r = Err(err)?;
    Ok(r)
}

fn fail_directly<'fail, 'o, 'f>(err: Failure<'fail>) -> StdResult<'o, 'f>
where
    'fail: 'f,
{
    Err(err)
}

fn pass<'pass, 'o, 'f>(val: &'pass i32) -> StdResult<'o, 'f>
where
    'pass: 'o,
{
    Ok(val)
}

impl<'t, 'e, 'f, E> From<BorrowedResult<'t, 'e, !, E>> for Failure<'f>
where
    'e: 'f,
    &'e E: Into<&'f i32>,
{
    fn from(res: BorrowedResult<'t, 'e, !, E>) -> Self {
        match res {
            BorrowedResult::Err(e) => Failure(e.into()),
            BorrowedResult::Ok(&never) => match never {},
        }
    }
}

impl<'t, 'e, 'f, T> From<Failure<'f>> for BorrowedResult<'t, 'e, T, i32>
where
    'f: 'e,
{
    fn from(f: Failure<'f>) -> Self {
        BorrowedResult::Err(f.0)
    }
}

fn _unrestricted_t_borrowed_to_result<'t, 'e, 'o, 'f>(
    okval: &'t i32,
    errval: &'e i32,
) -> StdResult<'o, 'f>
where
    'e: 'f,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::pass(okval)?,
        5 => BorrowedResult::fail(errval)?,
        6.. => BorrowedResult::fail_directly(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_e_borrowed_to_result_direct<'t, 'e, 'o, 'f>(
    okval: &'t i32,
    errval: &'e i32,
) -> StdResult<'o, 'f>
where
    't: 'o,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::pass(okval)?,
        5.. => BorrowedResult::fail_directly(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_e_borrowed_to_result_indirect<'t, 'e, 'o, 'f>(
    okval: &'t i32,
    errval: &'e i32,
) -> StdResult<'o, 'f>
where
    't: 'o,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::pass(okval)?,
        5.. => BorrowedResult::fail(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_t_result_to_borrowed<'o, 'f, 't, 'e>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'f: 'e,
{
    let rtn = match errval {
        ..=4 => pass(okval)?,
        5 => fail(Failure(errval))?,
        6.. => fail_directly(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn _unrestricted_e_result_to_borrowed_direct<'o, 'f, 't, 'e>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'o: 't,
{
    let rtn = match errval {
        ..=4 => pass(okval)?,
        5.. => fail_directly(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn _unrestricted_e_result_to_borrowed_indirect<'o, 'f, 't, 'e>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'o: 't,
{
    let rtn = match errval {
        ..=4 => pass(okval)?,
        5.. => fail(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn main() {}
