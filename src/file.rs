use std::borrow::Cow;
use std::fs;
use std::io::{self, Cursor, Read};

enum FileInner<'a> {
    Ram(Cursor<Cow<'a, [u8]>>),
    Fs(fs::File),
}

/// File you can Read from.
pub struct File<'a>(FileInner<'a>);

impl<'a> File<'a> {
    pub(crate) fn from_fs(file: fs::File) -> Self {
        File(FileInner::Fs(file))
    }

    pub(crate) fn from_ram<T: Into<Cow<'a, [u8]>>>(ram: T) -> Self {
        File(FileInner::Ram(Cursor::new(ram.into())))
    }
}

impl<'a> Read for File<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> Read for FileInner<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            FileInner::Ram(ram) => ram.read(buf),
            FileInner::Fs(fs) => fs.read(buf),
        }
    }
}
