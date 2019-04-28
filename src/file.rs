use crate::err::Result;
use std::borrow::Cow;
use std::fs;
use std::io::{self, Cursor, Read, Write};

enum FileInner<'a> {
    Ram(Cursor<Cow<'a, [u8]>>),
    Fs(fs::File),
}

/// File you can Read from.
pub struct File<'a> {
    inner: FileInner<'a>,
}

/// File you can Write to.
pub struct FileMut<'a> {
    inner: FileInner<'a>,
}

impl<'a> File<'a> {
    pub(crate) fn from_fs(file: fs::File) -> Self {
        File {
            inner: FileInner::Fs(file),
        }
    }

    pub(crate) fn from_ram<T: Into<Cow<'a, [u8]>>>(ram: T) -> Self {
        File {
            inner: FileInner::Ram(Cursor::new(ram.into())),
        }
    }

    pub fn as_std(&self) -> Option<&fs::File> {
        match &self.inner {
            FileInner::Fs(ref file) => Some(file),
            _ => None,
        }
    }

    /// Returns the internally wrapped file.
    pub fn into_std(self) -> Option<fs::File> {
        match self.inner {
            FileInner::Fs(file) => Some(file),
            _ => None,
        }
    }
}

impl<'a> FileMut<'a> {
    pub(crate) fn from_fs(file: fs::File) -> Self {
        FileMut {
            inner: FileInner::Fs(file),
        }
    }

    pub fn as_std_mut(&mut self) -> Option<&mut fs::File> {
        match &mut self.inner {
            FileInner::Fs(ref mut file) => Some(file),
            _ => None,
        }
    }

    pub fn as_std(&self) -> Option<&fs::File> {
        match &self.inner {
            FileInner::Fs(ref file) => Some(file),
            _ => None,
        }
    }

    pub fn into_std(self) -> Option<fs::File> {
        match self.inner {
            FileInner::Fs(file) => Some(file),
            _ => None,
        }
    }
}

impl<'a> Read for File<'a> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.inner {
            FileInner::Ram(ram) => ram.read(buf),
            FileInner::Fs(fs) => fs.read(buf),
        }
    }
}

impl<'a> Write for FileMut<'a> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.inner {
            FileInner::Fs(fs) => fs.write(buf),
            _ => Err(write_support_err()),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match &mut self.inner {
            FileInner::Fs(fs) => fs.flush(),
            _ => Err(write_support_err()),
        }
    }
}

pub fn write_support_err() -> io::Error {
    io::Error::new(io::ErrorKind::Other, "Operation not supported.")
}
