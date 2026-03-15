#![feature(assert_matches)]
#![feature(never_type)]
#![feature(try_trait_v2)]

use std::process::Termination;
use std::assert_matches::assert_matches;

use try_v2::Try;

#[derive(Debug, Try)]
enum Exit<T: Termination> {
    Ok(T),
    TestsFailed,
}

#[test]
fn short_circuit() {
    fn fail() -> Exit<()> {
        Exit::TestsFailed?;
        Exit::Ok(())
    }
    assert_matches!(fail(), Exit::TestsFailed)
}

#[test]
fn no_short_circuit() {
    fn pass() -> Exit<()> {
        Exit::Ok(())?;
        Exit::Ok(())
    }
    assert_matches!(pass(), Exit::Ok(()))
}