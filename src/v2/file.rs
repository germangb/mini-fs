use std::borrow::Cow;
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::rc::Rc;

use crate::err::Error;

enum FileInner {
    Ram(Cursor<Rc<[u8]>>),
    Fs(fs::File),
}

/// Concrete file type.
pub struct File(FileInner);

impl File {
    pub fn from_ram<T: Into<Rc<[u8]>>>(file: T) -> Self {
        File(FileInner::Ram(Cursor::new(file.into())))
    }

    pub fn from_std(file: fs::File) -> Self {
        File(FileInner::Fs(file))
    }

    pub fn into_std(self) -> Option<fs::File> {
        match self.0 {
            FileInner::Fs(file) => Some(file),
            _ => None,
        }
    }

    pub fn into_ram(self) -> Option<Rc<[u8]>> {
        match self.0 {
            FileInner::Ram(cursor) => Some(cursor.into_inner()),
            _ => None,
        }
    }
}

impl Read for File {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0 {
            FileInner::Ram(ref mut ram) => ram.read(buf),
            FileInner::Fs(ref mut file) => file.read(buf),
        }
    }
}

impl Write for File {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0 {
            FileInner::Ram(ref mut ram) => {
                Err(io::Error::new(io::ErrorKind::Other, "Write not supported."))
            }
            FileInner::Fs(ref mut file) => file.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match self.0 {
            FileInner::Ram(ref mut ram) => {
                Err(io::Error::new(io::ErrorKind::Other, "Write not supported."))
            }
            FileInner::Fs(ref mut file) => file.flush(),
        }
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self.0 {
            FileInner::Ram(ref mut ram) => ram.seek(pos),
            FileInner::Fs(ref mut file) => file.seek(pos),
        }
    }
}
