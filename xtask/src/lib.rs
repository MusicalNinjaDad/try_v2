use std::{
    io,
    path::Path,
    process::{Command, Output},
};

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

pub fn fmt(root: &Path) -> Result<Cmd, io::Error> {
    Command::new("cargo")
        .current_dir(root)
        .arg("fmt")
        .output()
        .map_into_cmd("fmt")
}

pub fn git_add(root: &Path) -> Result<Cmd, io::Error> {
    Command::new("git")
        .current_dir(root)
        .arg("add")
        .arg(".")
        .output()
        .map_into_cmd("git add")
}
