#![allow(unused_variables)]

use std::path::Path;

use mini_fs::prelude::*;
use mini_fs::{LocalFs, MiniFs, TarFs};

fn main() {
    let pwd = LocalFs::pwd().unwrap();
    let tar = TarFs::open("tests/archive.tar.gz").unwrap();

    let fs = MiniFs::new().mount("/a", (pwd, tar));

    fs.open("/a/Cargo.toml").unwrap();
}
