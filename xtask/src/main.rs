#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{io, path::Path, process::Termination as _T};

use clap::{Parser, Subcommand};
use exit_safely::Termination;
use try_v2::{Try, Try_ConvertResult};
use try_v2_xtasks::{Cmd, Spawned, clippy, clippy_tests, fmt, git_add, test};

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

impl From<Cmd> for Exit<()> {
    fn from(cmd: Cmd) -> Self {
        if cmd.output.status.success() {
            println!("{}: OK", cmd.name);
            Self::Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&cmd.output.stderr);
            Self::Error(stderr.to_string())
        }
    }
}

impl From<Vec<Spawned>> for Exit<()> {
    fn from(spawns: Vec<Spawned>) -> Self {
        let cmds: Vec<_> = spawns
            .into_iter()
            .map(|spawn| spawn.wait())
            .collect::<Result<Vec<_>, _>>()?;
        let errors: String = cmds
            .into_iter()
            .filter_map(|cmd| match Exit::from(cmd) {
                Self::Ok(_) => None,
                Self::Error(s) => Some(s + "\n"),
                _ => unreachable!("cmd always goes to Error"),
            })
            .collect();
        if errors.is_empty() {
            Self::Ok(())
        } else {
            Self::Error(errors)
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
            let clippy = clippy(root)?;
            let clippy_tests = clippy_tests(root)?;
            let tests = test(root)?;
            let checks = vec![clippy, clippy_tests, tests];
            Exit::from(checks)?;
            let git = git_add(root)?;
            Exit::from(git)
        }
    }
}
