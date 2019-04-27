use std::fs::File;
use std::io::Read;
use std::path::Path;

use mini_fs::{Local, MiniFs, Store, Tar};

fn main() {
    let fs = MiniFs::new()
        .mount("/local", Local::pwd().unwrap())
        .mount("/tar", Tar::open("tests/archive.tar").unwrap());

    let file = fs.open(Path::new("/tar/a.txt")).unwrap();
}
