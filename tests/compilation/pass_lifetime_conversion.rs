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

type StdResult<'o, 'f> = std::result::Result<&'o i32, Failure<'f>>;

#[derive(Debug, PartialEq, Eq)]
struct Failure<'f>(&'f i32);

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

fn borrowed_to_result_passthrough<'t, 'e>(okval: &'t i32, errval: &'e i32) -> StdResult<'t, 'e> {
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn borrowed_to_result_restricted<'t, 'e, 'o, 'f>(
    okval: &'t i32,
    errval: &'e i32,
) -> StdResult<'o, 'f>
where
    't: 'o,
    'e: 'f,
{
    let rtn = match errval {
        ..=4 => BorrowedResult::Ok(okval)?,
        5 => BorrowedResult::Err(errval)?,
        6.. => BorrowedResult::Err(errval)?,
    };
    Ok(rtn)
}

fn result_to_borrowed_passthrough<'o, 'f>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'o, 'f, i32, i32> {
    let rtn = match errval {
        ..=4 => Ok::<_, Failure<'f>>(okval)?,
        5 => Err(Failure(errval))?,
        6.. => Err(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn result_to_borrowed_restricted<'o, 'f, 't, 'e>(
    okval: &'o i32,
    errval: &'f i32,
) -> BorrowedResult<'t, 'e, i32, i32>
where
    'o: 't,
    'f: 'e,
{
    let rtn = match errval {
        ..=4 => Ok::<_, Failure<'f>>(okval)?,
        5 => Err(Failure(errval))?,
        6.. => Err(Failure(errval))?,
    };
    BorrowedResult::Ok(rtn)
}

fn main() {
    assert_matches!(borrowed_to_result_passthrough(&0, &1), StdResult::Ok(&0));
    assert_matches!(
        borrowed_to_result_passthrough(&0, &5),
        StdResult::Err(Failure(&5))
    );
    assert_matches!(
        borrowed_to_result_passthrough(&0, &7),
        StdResult::Err(Failure(&7))
    );

    assert_matches!(borrowed_to_result_restricted(&0, &1), StdResult::Ok(&0));
    assert_matches!(
        borrowed_to_result_restricted(&0, &5),
        StdResult::Err(Failure(&5))
    );
    assert_matches!(
        borrowed_to_result_restricted(&0, &7),
        StdResult::Err(Failure(&7))
    );

    assert_matches!(
        result_to_borrowed_passthrough(&0, &1),
        BorrowedResult::Ok(&0)
    );
    assert_matches!(
        result_to_borrowed_passthrough(&0, &5),
        BorrowedResult::Err(&5)
    );
    assert_matches!(
        result_to_borrowed_passthrough(&0, &7),
        BorrowedResult::Err(&7)
    );
    assert_matches!(
        result_to_borrowed_restricted(&0, &1),
        BorrowedResult::Ok(&0)
    );
    assert_matches!(
        result_to_borrowed_restricted(&0, &5),
        BorrowedResult::Err(&5)
    );
    assert_matches!(
        result_to_borrowed_restricted(&0, &7),
        BorrowedResult::Err(&7)
    );
}
