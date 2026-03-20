#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]

use std::assert_matches::assert_matches;
use std::process::Termination;

use try_v2::{Try, Try_ConvertResult};

#[derive(Debug, Try, Try_ConvertResult)]
enum Exit<T: Termination> {
    Ok(T),
    TestsFailed,
    OtherError(String),
}

#[derive(Debug)]
struct AnError(String);

#[derive(Debug, Try, Try_ConvertResult)]
#[allow(unused)] // If it compiles then it already passes
enum NoFieldResiduals<T: Termination> {
    Ok(T),
    TestsFailed,
}

#[derive(Debug, Try, Try_ConvertResult)]
#[allow(unused)] // If it compiles then it already passes
enum NoUnitResiduals<T: Termination> {
    Ok(T),
    OtherError(String),
}

#[derive(Debug, Try, Try_ConvertResult)]
enum ExitE<E> {
    Ok(E),
    TestsFailed,
    OtherError(String),
}

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

impl From<Exit<!>> for AnError {
    fn from(exit: Exit<!>) -> Self {
        match exit {
            Exit::TestsFailed => AnError("tests failed".to_string()),
            Exit::OtherError(text) => AnError(text),
        }
    }
}
