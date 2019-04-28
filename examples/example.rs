use std::fs::File;
use std::io::Read;
use std::path::Path;

use mini_fs::{Local, MiniFs, Store, Tar};

fn main() {
    let home = Local::new("/home/germangb");
    let pwd = Local::pwd().unwrap();
    let tar = Tar::open("tests/archive.tar").unwrap();

    let fs = MiniFs::new()
        .mount("/local", (home, pwd))
        .mount("/tar", tar);

    let file = fs.open(Path::new("/tar/a.txt")).unwrap();
    let file = fs.open(Path::new("/local/.bashrc")).unwrap();
    let file = fs.open(Path::new("/local/src/../Cargo.toml")).unwrap();
}
