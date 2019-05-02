use mini_fs::{MiniFs, Store, UserFile};
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::path::Path;

// Virtual file system with a single "hello.txt" file.
struct HelloWorld;
struct File(Cursor<&'static str>);

// Implement storage
impl Store for HelloWorld {
    type File = File;
    fn open_path(&self, path: &Path) -> Result<Self::File> {
        if path == Path::new("hello.txt") {
            Ok(File(Cursor::new("hello world!")))
        } else {
            Err(Error::from(ErrorKind::NotFound))
        }
    }
}

// Implement IO on File.
impl UserFile for File {}
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.0.seek(pos)
    }
}

fn main() {
    let fs = MiniFs::new().mount("/files", HelloWorld);

    let mut s = String::new();

    fs.open("/files/hello.txt")
        .and_then(|mut file| file.read_to_string(&mut s))
        .unwrap();

    println!("{}", s); // hello world!
}
