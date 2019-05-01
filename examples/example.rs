use std::io::Read;
use std::path::Path;

use mini_fs::v2::file::File;
use mini_fs::v2::{Local, MiniFs};

fn main() {
    let pwd = Local::pwd().unwrap();

    let fs = MiniFs::new().mount("/local", pwd);
}
