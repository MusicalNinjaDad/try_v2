#![feature(never_type)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use std::{
    fmt::Debug,
    io,
    process::{Child, Output, Termination as _T},
};

use exit_safely::Termination;
use try_v2::{Try, Try_ConvertResult};

pub mod commands;

#[derive(Debug)]
pub struct Cmd {
    pub name: &'static str,
    pub output: Output,
}

#[derive(Debug)]
pub struct Cmd_ {
    pub name: &'static str,
    pub result: Result<Output, io::Error>,
}

trait CmdExt {
    fn map_into_cmd(self, name: &'static str) -> Result<Cmd, io::Error>;
    fn into_cmd(self, name: &'static str) -> Cmd_;
}

impl CmdExt for Result<Output, io::Error> {
    fn map_into_cmd(self, name: &'static str) -> Result<Cmd, io::Error> {
        self.map(|output| Cmd { name, output })
    }

    fn into_cmd(self, name: &'static str) -> Cmd_ {
        Cmd_ { name, result: self }
    }
}

#[derive(Debug)]
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
    InvocationError(String) = 2,
    IO(String) = 3,
    IO2(String) = 9,
}

impl FromIterator<Exit<()>> for Exit<()> {
    fn from_iter<I: IntoIterator<Item = Exit<()>>>(iter: I) -> Self {
        let mut s = String::new();
        let mut errnos: Vec<u8> = vec![];
        for e in iter {
            match e {
                Exit::Ok(_) => {},
                Exit::Error(ref m) => {
                    s.push_str(m);
                    errnos.push(1);
                }
                Exit::InvocationError(ref error) => {
                    s.push_str(error.to_string().as_str());
                    errnos.push(2);
                }
                Exit::IO(ref error) => {
                    s.push_str(error.to_string().as_str());
                    errnos.push(3);
                }
                Exit::IO2(ref m) => {
                    s.push_str(m);
                    errnos.push(9);
                }
            };
        }
        match errnos.into_iter().min().expect("at least one exit") {
            0 => todo!(),
            1 => Exit::Error(s),
            2 => todo!(),
            3 => todo!(),
            9 => Exit::IO2(s),
            _ => todo!(),
        }
    }
}

impl<T: _T> From<clap::Error> for Exit<T> {
    fn from(e: clap::Error) -> Self {
        Self::InvocationError(e.to_string())
    }
}

impl<T: _T> From<io::Error> for Exit<T> {
    fn from(e: io::Error) -> Self {
        Self::IO(e.to_string())
    }
}

impl From<Cmd_> for Exit<()> {
    fn from(cmd: Cmd_) -> Self {
        match cmd.result {
            Ok(output) => {
                if output.status.success() {
                    println!("{}: OK", cmd.name);
                    Self::Ok(())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Self::Error(stderr.to_string())
                }
            }
            Err(e) => {
                let msg = format!("{} failed: {}", cmd.name, e);
                Self::IO2(msg)
            }
        }
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

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    #[test]
    fn exit_from_404() {
        let splat: Cmd_ = Command::new("splat").output().into_cmd("splat");
        assert_eq!(splat.name, "splat");
        assert!(
            matches!(splat.result, Result::Err(ref e) if matches!(e.kind(), io::ErrorKind::NotFound))
        );
        let exit: Exit<()> = Exit::from(splat);
        let Exit::IO2(ref msg) = exit else {
            panic!("not an IO2")
        };
        eprintln!("{}", msg);
        assert!(msg.starts_with("splat failed: "));
    }

    #[test]
    fn collect_exit() {
        let exits = [
            Exit::Ok(()),
            Exit::Error("one".to_string()),
            Exit::IO2("two".to_string()),
            Exit::Error("three".to_string()),
        ];
        let exit: Exit<()> = exits.into_iter().collect();
        let expected = "one\ntwo\nthree\n";
        dbg!(&exit);
        assert!(matches!(exit, Exit::IO2(s) if s == expected));
    }
}
