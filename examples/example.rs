#![allow(unused_variables)]

use std::path::Path;

use mini_fs::tar::TarEntry;
use mini_fs::{Local, MiniFs, Store, Tar};

fn main() {
    let pwd = Local::pwd().unwrap();
    let tar = Tar::open("tests/archive.tar.gz").unwrap();

    let fs = MiniFs::new().mount("/a", (pwd, tar));

    fs.open("/a/Cargo.toml").unwrap();
}
