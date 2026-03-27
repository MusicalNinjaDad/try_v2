#![feature(never_type)]
#![feature(try_trait_v2)]

//! Tests that conversion between custom enum and std::result::Result requires correct lifetime bounds.

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum BorrowedResult<'t, 'e, T, E> {
    Ok(&'t T),
    Err(&'e E),
}

type StdResult<'o, 'f> = std::result::Result<&'o i32, Failure<'f>>;

#[derive(Debug)]
struct Failure<'f>(&'f i32);

impl<'a, 'e, 'f, E> From<BorrowedResult<'a, 'e, !, E>> for Failure<'f>
where
    'e: 'f,
    &'e E: Into<&'f i32>,
{
    fn from(res: BorrowedResult<'a, 'e, !, E>) -> Self {
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
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn _unrestricted_e_borrowed_to_result<'t, 'e, 'o, 'f>(
    okval: &'t i32,
    errval: &'e i32,
) -> StdResult<'o, 'f>
where
    't: 'o,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
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
        ..=4 => Ok::<_, Failure<'f>>(okval)?,
        5 => Err(Failure(errval))?,
        6.. => Err(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn _unrestricted_e_result_to_borrowed<'o, 'f, 't, 'e>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'o: 't,
{
    let rtn = match errval {
        ..=4 => Ok::<_, Failure<'f>>(okval)?,
        5 => Err(Failure(errval))?,
        6.. => Err(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn main() {}
