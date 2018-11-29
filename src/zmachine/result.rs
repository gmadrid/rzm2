use std::io;
use std::result;

pub type Result<T> = result::Result<T, ZErr>;

#[derive(Debug)]
pub enum ZErr {
    UnknownVersionNumber(u8),

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
