use std::borrow::Cow;
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use crate::err::Error;

enum FileInner<'a> {
    Ram(Cursor<Cow<'a, [u8]>>),
    Fs(fs::File),
}

/// Concrete file type.
pub struct File<'a>(FileInner<'a>);

impl<'a> File<'a> {
    pub fn from_ram<T: Into<Cow<'a, [u8]>>>(file: T) -> Self {
        File(FileInner::Ram(Cursor::new(file.into())))
    }

    pub fn from_std(file: fs::File) -> Self {
        File(FileInner::Fs(file))
    }
}

impl<'a> Read for File<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
}

impl<'a> Write for File<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
}

impl<'a> Seek for File<'a> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        unimplemented!()
    }
}
