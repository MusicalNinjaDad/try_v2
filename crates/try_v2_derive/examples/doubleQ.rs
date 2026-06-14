#![allow(unused)]
#![allow(clippy::disallowed_names)]
use std::io;

fn wibble() -> io::Result<Option<()>> {
    // let _ = Ok(Some(3))??;
    Ok(Some(()))
}

fn main() {}
