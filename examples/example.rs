#![allow(unused_variables)]

use std::path::Path;

use mini_fs::{Local, MiniFs};

fn main() {
    let pwd = Local::pwd().unwrap();
    let tmp = Local::new("/tmp/override");

    let fs = MiniFs::new().mount("/local", (tmp, pwd));
}
