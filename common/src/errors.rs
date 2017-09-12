use std::io;
use std::str;
use std::mem;

#[cfg(feature="with_dwarf")]
use gimli;
#[cfg(feature="with_objects")]
use goblin;
#[cfg(feature="with_objects")]
use scroll;

error_chain! {
    errors {
        /// Returned on operations that work on symbols if the symbol
        /// data is bad.
        BadSymbol(message: String) {
            description("bad symbol")
            display("bad symbol: {}", &message)
        }
        /// Raised for internal errors in the libraries.  Should not happen.
        InternalError(message: &'static str) {
            description("internal error")
            display("internal error: {}", &message)
        }
        /// Raised for bad input on parsing in symbolic.
        ParseError(message: &'static str) {
            description("parse error")
            display("parse error: {}", &message)
        }

        /// Returned if unsupported object files are loaded.
        UnsupportedObjectFile {
            description("unsupported object file")
        }
        /// Returned if object files are malformed.
        MalformedObjectFile(msg: String) {
            description("malformed object file")
            display("malformed object file: {}", &msg)
        }
        /// Returned for unknown cache file versions.
        UnknownCacheFileVersion(version: u32) {
            description("unknown cache file version")
            display("unknown cache file version '{}'", version)
        }
        /// Returned for DWARF failures.
        BadDwarfData(msg: &'static str) {
            description("bad dwarf data")
            display("bad dwarf data: {}", msg)
        }
    }

    foreign_links {
        Io(io::Error);
        Utf8Error(str::Utf8Error);
    }
}

#[cfg(feature="with_dwarf")]
impl From<gimli::Error> for Error {
    fn from(err: gimli::Error) -> Error {
        use std::error::Error;
        // this works because gimli error only returns static strings. UUUGLY
        ErrorKind::BadDwarfData(unsafe {
            mem::transmute(err.description())
        }).into()
    }
}

#[cfg(feature="with_objects")]
impl From<goblin::error::Error> for Error {
    fn from(err: goblin::error::Error) -> Error {
        use goblin::error::Error::*;
        match err {
            Malformed(s) => ErrorKind::MalformedObjectFile(s).into(),
            BadMagic(m) => ErrorKind::MalformedObjectFile(format!("bad magic: {}", m)).into(),
            Scroll(err) => Error::from(err),
            IO(err) => Error::from(err),
        }
    }
}

#[cfg(feature="with_objects")]
impl From<scroll::Error> for Error {
    fn from(err: scroll::Error) -> Error {
        use scroll::Error::*;
        match err {
            TooBig { .. } => io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Tried to read type that was too large",
            ).into(),
            BadOffset(..) => io::Error::new(io::ErrorKind::InvalidData, "Bad offset").into(),
            BadInput { .. } => io::Error::new(io::ErrorKind::InvalidData, "Bad input").into(),
            Custom(s) => io::Error::new(io::ErrorKind::Other, s).into(),
            IO(err) => Error::from(err),
        }
    }
}
