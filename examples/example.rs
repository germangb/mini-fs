use std::io::Read;
use std::path::Path;

use mini_fs::err::Error;
use mini_fs::{Local, MiniFs, Store, Tar};
use std::fs::File;

fn main() -> Result<(), Error> {
    let a = Local::pwd()?;
    let b = Tar::open("archive.tar")?;
    let mut fs = MiniFs::new().mount("/local", a).mount("/tar", b);

    fs.umount("/local");

    let mut content = String::new();
    for _ in 0..2 {
        content.clear();

        let mut file = fs.open(Path::new("/tar/a.txt"))?;
        file.read_to_string(&mut content)?;

        println!("{}", content);
    }

    Ok(())
}
