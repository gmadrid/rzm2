use std::io;
use std::result;

pub type Result<T> = result::Result<T, ZErr>;

#[derive(Debug)]
pub enum ZErr {
    UnknownVersionNumber(u8),

    IO(io::Error),
}

impl From<io::Error> for ZErr {
    fn from(err: io::Error) -> ZErr {
        ZErr::IO(err)
    }
}
