use std::fmt;
use std::io;
use std::result;

pub type Result<T> = result::Result<T, ZErr>;

#[derive(Debug)]
pub enum ZErr {
    UnknownVersionNumber(u8),
    WriteViolation(usize),

    IO(io::Error),
}

pub trait ToTrue {
    fn to_true(self) -> Result<bool>;
}

impl ToTrue for Result<()> {
    fn to_true(self) -> Result<bool> {
        self.map(|_| true)
    }
}

impl From<io::Error> for ZErr {
    fn from(err: io::Error) -> ZErr {
        ZErr::IO(err)
    }
}

impl fmt::Display for ZErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ZErr::*;
        match *self {
            UnknownVersionNumber(vers) => write!(f, "Unknown version number: '{}'", vers),
            WriteViolation(offset) => write!(
                f,
                "Attempt to write to read-only memory at offset '{}'",
                offset
            ),

            // Wrapped errors.
            IO(ref io_error) => io_error.fmt(f),
        }
    }
}
