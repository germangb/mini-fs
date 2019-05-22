macro_rules! impl_file {
    (
        $(#[$($meta:meta)+])*
        pub enum $enum_name:ident {
            $(
                $(#[$($var_meta:meta)+])*
                $var_name:ident($var_type:ty),
            )*
        }
    ) => {
        $(#[$($meta)+])*
        pub enum $enum_name {
            $(
                $(#[$($var_meta)+])*
                $var_name($var_type),
            )*
        }

        $(
            $(#[$($var_meta)+])*
            impl From<$var_type> for $enum_name {
                fn from(file: $var_type) -> Self {
                    $enum_name::$var_name(file)
                }
            }
        )*

        impl std::io::Read for $enum_name {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                match self {
                    $(
                        $(#[$($var_meta)+])*
                        $enum_name::$var_name(ref mut file) => file.read(buf),
                    )*
                }
            }
        }

        impl std::io::Seek for File {
            fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
                match self {
                    $(
                        $(#[$($var_meta)+])*
                        $enum_name::$var_name(ref mut file) => file.seek(pos),
                    )*
                }
            }
        }
    }
}

macro_rules! store_entries {
    ($self:ident, $path:expr, $head:ident,) => { $self.0.entries_path($path)? };
    ($self:ident, $path:expr, $head:ident, $($tail:ident,)+) => {
        $self.0.entries_path($path)?.chain(store_entries!($self, $path, $($tail,)+) )
    }
}

macro_rules! store_tuples {
    ($head:ident,) => {};
    ($head:ident, $($tail:ident,)+) => {
        impl<$head, $($tail,)+> Store for ($head, $($tail,)+)
        where
            $head: Store,
            $($tail: Store,)+
            $head::File: Into<$crate::File>,
            $($tail::File: Into<$crate::File>,)+
        {
            type File = $crate::File;
            #[allow(non_snake_case)]
            fn open_path(&self, path: &Path) -> io::Result<Self::File> {
                let ($head, $($tail,)+) = self;
                match $head.open_path(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                $(
                match $tail.open_path(path) {
                    Ok(file) => return Ok(file.into()),
                    Err(ref err) if err.kind() == io::ErrorKind::NotFound => {},
                    Err(err) => return Err(err),
                }
                )+

                Err(io::Error::from(io::ErrorKind::NotFound))
            }

            fn entries_path(&self, path: &Path) -> io::Result<Entries> {
                // chain all elements from the tuple
                let raw = store_entries!(self, path, $head, $($tail,)+);
                Ok(Entries::new(TupleEntries::new(raw)))
            }
        }
        store_tuples!($($tail,)+);
    };
}

