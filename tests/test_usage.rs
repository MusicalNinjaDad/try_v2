#![cfg_attr(unstable_assert_matches, feature(assert_matches))]
#![feature(iterator_try_collect)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use try_v2::{Try, Try_ConvertResult, Try_Methods};

#[cfg(assert_matches_in_module)]
use std::assert_matches::assert_matches;

#[cfg(assert_matches_in_root)]
use std::assert_matches;

mod bound_ok_type {
    use super::*;

    use std::process::Termination;

    #[derive(Debug, Try, Try_ConvertResult)]
    #[must_use]
    enum Exit<T: Termination> {
        Ok(T),
        TestsFailed,
        OtherError(String),
        NumberedError(String, i32),
        FormalError { errno: i32, data: String },
    }

    impl From<Exit<!>> for AnError {
        fn from(exit: Exit<!>) -> Self {
            match exit {
                Exit::TestsFailed => AnError("tests failed".to_string()),
                Exit::OtherError(text) => AnError(text),
                Exit::NumberedError(text, n) => AnError(format!("{n}: {text}")),
                Exit::FormalError { errno, data } => AnError(format!("{errno}: {data}")),
            }
        }
    }
    #[derive(Debug)]
    struct AnError(String);

    #[test]
    fn short_circuit_1() {
        fn fail() -> Exit<()> {
            Exit::TestsFailed?;
            Exit::Ok(())
        }
        assert_matches!(fail(), Exit::TestsFailed)
    }

    #[test]
    fn short_circuit_2() {
        fn fail() -> Exit<()> {
            Exit::OtherError("oops!".to_string())?;
            Exit::TestsFailed?;
            Exit::Ok(())
        }
        assert_matches!(fail(), Exit::OtherError(msg) if msg == "oops!")
    }

    #[test]
    fn short_circuit_3() {
        fn fail() -> Exit<()> {
            Exit::NumberedError("oops!".to_string(), 4)?;
            Exit::TestsFailed?;
            Exit::Ok(())
        }
        assert_matches!(fail(), Exit::NumberedError(msg, n) if msg == "oops!" && n == 4)
    }

    #[test]
    fn short_circuit_4() {
        fn fail() -> Exit<()> {
            Exit::FormalError {
                errno: 2,
                data: "oops!".to_string(),
            }?;
            Exit::TestsFailed?;
            Exit::Ok(())
        }
        assert_matches!(fail(), Exit::FormalError{errno, data} if data == "oops!" && errno == 2)
    }

    #[test]
    fn no_short_circuit() {
        fn pass() -> Exit<()> {
            Exit::Ok(())?;
            Exit::Ok(())
        }
        assert_matches!(pass(), Exit::Ok(()))
    }

    #[test]
    fn convert_to_result_1() {
        fn fail() -> Result<(), AnError> {
            Exit::TestsFailed?;
            Ok(())
        }
        assert_matches!(fail(), Result::Err(e) if e.0 == "tests failed")
    }

    #[test]
    fn convert_to_result_2() {
        fn fail() -> Result<(), AnError> {
            Exit::OtherError("oops!".to_string())?;
            Exit::TestsFailed?;
            Ok(())
        }
        assert_matches!(fail(), Result::Err(e) if e.0 == "oops!")
    }
}

mod multiple_generics {
    use super::*;

    #[derive(Debug, Try, Try_ConvertResult)]
    #[must_use]
    enum MyResult<T, E> {
        Ok(T),
        Err(E),
    }

    #[test]
    fn short_circuit_ok() {
        fn pass() -> MyResult<usize, String> {
            let foo = MyResult::Ok(5)?;
            MyResult::Ok(foo)
        }
        assert_matches!(pass(), MyResult::Ok(val) if val == 5);
    }

    #[test]
    fn short_circuit_fail() {
        fn fail() -> MyResult<String, String> {
            let foo = MyResult::Err("oops!".to_string())?;
            MyResult::Ok(foo)
        }
        assert_matches!(fail(), MyResult::Err(msg) if msg == "oops!");
    }
}

mod iter {
    use try_v2::Try_Iterator;

    use super::*;

    #[derive(Debug, Try, Try_Iterator)]
    #[must_use]
    enum MyResult<T> {
        Ok(T),
        Err,
    }

    #[test]
    fn not_copy() {
        let mut res: MyResult<String> = MyResult::Ok("String is not Copy".to_string());
        let borrowed_text: &String = res.iter().next().unwrap();
        assert_eq!(borrowed_text, "String is not Copy");
        if let Some(text) = res.iter_mut().next() {
            *text = "Another String".to_string();
        };
        let text: String = res.into_iter().next().unwrap();
        assert_eq!(text, "Another String");
    }
}

// TODO: #62 fix tests to exercise non-trivial lifetime relationships
mod lifetime_conversion {

    use super::*;

    #[derive(Debug, Try, Try_ConvertResult)]
    #[must_use]
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

    #[test]
    fn test_borrowed_to_result_passthrough() {
        fn borrowed_to_result_passthrough<'t, 'e>(
            okval: &'t i32,
            errval: &'e i32,
        ) -> StdResult<'t, 'e> {
            let rtn = match errval {
                ..=4 => BorrowedResult::pass(okval)?,
                5 => BorrowedResult::fail(errval)?,
                6.. => BorrowedResult::fail_directly(errval)?,
            };
            Ok(rtn)
        }

        assert_matches!(borrowed_to_result_passthrough(&0, &1), StdResult::Ok(&0));
        assert_matches!(
            borrowed_to_result_passthrough(&0, &5),
            StdResult::Err(Failure(&5))
        );
        assert_matches!(
            borrowed_to_result_passthrough(&0, &7),
            StdResult::Err(Failure(&7))
        );
    }

    #[test]
    fn test_borrowed_to_result_restricted() {
        fn borrowed_to_result_restricted<'t, 'e, 'o, 'f>(
            okval: &'t i32,
            errval: &'e i32,
        ) -> StdResult<'o, 'f>
        where
            't: 'o,
            'e: 'f,
        {
            let rtn = match errval {
                ..=4 => BorrowedResult::pass(okval)?,
                5 => BorrowedResult::fail(errval)?,
                6.. => BorrowedResult::fail_directly(errval)?,
            };
            Ok(rtn)
        }

        assert_matches!(borrowed_to_result_restricted(&0, &1), StdResult::Ok(&0));
        assert_matches!(
            borrowed_to_result_restricted(&0, &5),
            StdResult::Err(Failure(&5))
        );
        assert_matches!(
            borrowed_to_result_restricted(&0, &7),
            StdResult::Err(Failure(&7))
        );
    }

    #[test]
    fn test_result_to_borrowed_passthrough() {
        fn result_to_borrowed_passthrough<'o, 'f>(
            okval: &'o i32,
            errval: &'f i32,
        ) -> BorrowedResult<'o, 'f, i32, i32> {
            let rtn = match errval {
                ..=4 => pass(okval)?,
                5 => fail(Failure(errval))?,
                6.. => fail_directly(Failure(errval))?,
            };
            BorrowedResult::Ok(rtn)
        }

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
    }

    #[test]
    fn test_result_to_borrowed_restricted() {
        fn result_to_borrowed_restricted<'o, 'f, 't, 'e>(
            okval: &'o i32,
            errval: &'f i32,
        ) -> BorrowedResult<'t, 'e, i32, i32>
        where
            'o: 't,
            'f: 'e,
        {
            let rtn = match errval {
                ..=4 => pass(okval)?,
                5 => fail(Failure(errval))?,
                6.. => fail_directly(Failure(errval))?,
            };
            BorrowedResult::Ok(rtn)
        }

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
}

// TODO: #62 fix tests to exercise non-trivial lifetime relationships
mod lifetime_duration {
    use super::*;

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

    #[test]
    fn test_passthrough_lifetime() {
        fn passthrough_lifetime<'t, 'e>(
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

        assert_matches!(passthrough_lifetime(&0, &1), BorrowedResult::Ok(&0));
        assert_matches!(passthrough_lifetime(&0, &5), BorrowedResult::Err(&5));
        assert_matches!(passthrough_lifetime(&0, &7), BorrowedResult::Err(&7));
    }

    #[test]
    fn test_restricted_lifetimes() {
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

        assert_matches!(restricted_lifetimes(&0, &1), BorrowedResult::Ok(&0));
        assert_matches!(restricted_lifetimes(&0, &5), BorrowedResult::Err(&5));
        assert_matches!(restricted_lifetimes(&0, &7), BorrowedResult::Err(&7));
    }
}

/// Validate that the derived methods work
mod methods {

    use super::*;

    #[derive(Debug, Try, Try_Methods)]
    #[must_use]
    enum Validated<T, E> {
        Valid(T),
        Invalid,
        ValidationFailed(E),
    }

    #[test]
    fn unwrap() {
        let x: Validated<_, String> = Validated::Valid(2);
        assert_eq!(x.unwrap(), 2);
    }

    #[test]
    #[should_panic(expected = "called `unwrap()` on a short-circuiting value: ValidationFailed(2)")]
    fn unwrap_panic_fields() {
        let x: Validated<i32, _> = Validated::ValidationFailed(2);
        assert_eq!(x.unwrap(), 2);
    }

    #[test]
    #[should_panic(expected = "called `unwrap()` on a short-circuiting value: Invalid")]
    fn unwrap_panic_unit() {
        let y: Validated<i32, String> = Validated::Invalid;
        y.unwrap();
    }
}
