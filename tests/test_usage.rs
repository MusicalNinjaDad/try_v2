#![cfg_attr(not(stable_assert_matches), feature(assert_matches))]
#![feature(iterator_try_collect)]
#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

#[cfg(not(stable_assert_matches))]
use std::assert_matches::assert_matches;

#[cfg(stable_assert_matches)]
use std::assert_matches;

use try_v2::{Try, Try_ConvertResult};

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
