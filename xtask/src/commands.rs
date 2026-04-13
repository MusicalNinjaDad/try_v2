use std::{
    io,
    path::Path,
    process::{Command, Stdio},
};

use crate::{Cmd, CmdExt as _, Spawned, SpawnedExt as _};

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

pub fn clippy_tests(root: &Path) -> Result<Spawned, io::Error> {
    Command::new("cargo")
        .current_dir(root)
        .arg("clippy")
        .arg("--tests")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_into_spawned("clippy the tests")
}

pub fn test(root: &Path) -> Result<Spawned, io::Error> {
    Command::new("cargo")
        .current_dir(root)
        .arg("test")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_into_spawned("tests")
}
