use std::{
    io,
    path::Path,
    process::{Child, Command, Output, Stdio},
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

pub fn clippy(root: &Path) -> Result<Spawned, io::Error> {
    Command::new("cargo")
        .current_dir(root)
        .arg("clippy")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_into_spawned("clippy")
}
