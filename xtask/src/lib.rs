#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{
    io,
    process::{Child, Output, Termination as _T},
};

use exit_safely::Termination;
use try_v2::{Try, Try_ConvertResult};

pub mod commands;

pub struct Cmd {
    pub name: &'static str,
    pub output: Output,
}

trait CmdExt<E> {
    fn map_into_cmd(self, name: &'static str) -> Result<Cmd, E>;
}

impl<E> CmdExt<E> for Result<Output, E> {
    fn map_into_cmd(self, name: &'static str) -> Result<Cmd, E> {
        self.map(|output| Cmd { name, output })
    }
}

pub struct Spawned {
    pub name: &'static str,
    pub child: Child,
}

impl Spawned {
    pub fn wait(self) -> Result<Cmd, io::Error> {
        self.child.wait_with_output().map_into_cmd(self.name)
    }
}

trait SpawnedExt<E> {
    fn map_into_spawned(self, name: &'static str) -> Result<Spawned, E>;
}

impl<E> SpawnedExt<E> for Result<Child, E> {
    fn map_into_spawned(self, name: &'static str) -> Result<Spawned, E> {
        self.map(|child| Spawned { name, child })
    }
}

#[derive(Debug, Termination, Try, Try_ConvertResult)]
#[repr(u8)]
#[must_use]
pub enum Exit<T: _T> {
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
