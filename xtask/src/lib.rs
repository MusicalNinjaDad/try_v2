use std::{
    io,
    path::Path,
    process::{Command, Output},
};

pub fn fmt(root: &Path) -> Result<Output, io::Error> {
    Command::new("cargo").current_dir(root).arg("fmt").output()
}
