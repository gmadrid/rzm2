use std::fmt;
use std::io;
use std::result;

pub type Result<T> = result::Result<T, ZErr>;

#[derive(Debug)]
pub enum ZErr {
    BadVariableIndex(&'static str, u8),
    LocalOutOfRange(u8, u8), // Requested local, num_locals.
    MissingOperand,
    StackOverflow(&'static str),
    StackUnderflow(&'static str),
    UnknownOpcode(&'static str, u16),
    UnknownVersionNumber(u8),
    WriteViolation(usize),

    GenericError(&'static str),

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
            BadVariableIndex(msg, index) => write!(f, "Bad {} variable index: {}", msg, index),
            GenericError(msg) => write!(f, "Generic error: {}", msg),
            LocalOutOfRange(req, num) => write!(
                f,
                "Requested local at index {}, but only {} locals available in frame.",
                req, num
            ),
            MissingOperand => write!(f, "Missing operand."),
            StackOverflow(msg) => write!(f, "Stack overflow: {}", msg),
            StackUnderflow(msg) => write!(f, "Stack underflow: {}", msg),
            UnknownOpcode(msg, opcode) =>
                write!(f, "Unknown {} opcode: 0x{:02x}", msg, opcode),
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
