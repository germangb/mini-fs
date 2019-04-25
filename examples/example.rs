use std::io::Read;
use std::path::Path;

use mini_fs::err::Error;
use mini_fs::{Local, MiniFs, Store};

fn main() -> Result<(), Error> {
    let files = MiniFs::new()
        .mount("/local", Local::pwd()?)
        .mount("/local/mirror", Local::pwd()?)
        .build()?;

    let mut file = files.open(Path::new("/local/mirror/Cargo.toml"))?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    println!("{}", content);

    Ok(())
}
