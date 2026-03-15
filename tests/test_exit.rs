#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]

use std::assert_matches::assert_matches;
use std::process::Termination;

use try_v2::Try;

#[derive(Debug, Try)]
enum Exit<T: Termination> {
    Ok(T),
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
