#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{
    io,
    path::Path,
    process::{Output, Termination as _T},
};

use clap::{Parser, Subcommand};
use exit_safely::Termination;
use try_v2::{Try, Try_ConvertResult};
use try_v2_xtasks::{fmt, git_add};

#[derive(Debug, Termination, Try, Try_ConvertResult)]
#[repr(u8)]
#[must_use]
enum Exit<T: _T> {
    Ok(T) = 0,
    Error(String) = 1,
    InvocationError(Box<clap::Error>) = 2,
    IO(Box<io::Error>) = 3,
}

impl<T: _T> From<clap::Error> for Exit<T> {
    fn from(e: clap::Error) -> Self {
        Self::InvocationError(Box::new(e))
    }
}

impl<T: _T> From<io::Error> for Exit<T> {
    fn from(e: io::Error) -> Self {
        Self::IO(Box::new(e))
    }
}

impl From<Output> for Exit<()> {
    fn from(output: Output) -> Self {
        if output.status.success() {
            Self::Ok(())
        } else {
            let stderr: String = String::from_utf8(output.stderr).unwrap_or_default();
            Self::Error(stderr)
        }
    }
}

#[derive(Parser)]
#[command(version)]
struct XTask {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// git add if all is good
    Add,
}

fn main() -> Exit<()> {
    let xtask = XTask::try_parse()?;

    match &xtask.command {
        Command::Add => {
            let root = Path::new(".");
            let fmt = fmt(root)?;
            Exit::from(fmt)?;
            let git = git_add(root)?;
            Exit::from(git)
        }
    }
}
