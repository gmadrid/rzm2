use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ZVariable {
    Stack,
    Local(u8),  // 0..MAX_LOCAL
    Global(u8), // 0..MAX_GLOBAL
}

impl From<u8> for ZVariable {
    fn from(byte: u8) -> ZVariable {
        match byte {
            0 => ZVariable::Stack,
            1...0x0f => ZVariable::Local(byte - 1),
            0x10...0xff => ZVariable::Global(byte - 0x10),
            _ => panic!("The compiler made me do this."),
        }
    }
}

impl From<ZVariable> for u8 {
    fn from(var: ZVariable) -> u8 {
        match var {
            ZVariable::Stack => 0x00,
            ZVariable::Local(l) => l + 0x01,
            ZVariable::Global(g) => g + 0x10,
        }
    }
}

impl fmt::Display for ZVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ZVariable::*;
        match *self {
            Stack => write!(f, "sp"),
            Local(l) => write!(f, "l{:01x}", l),
            Global(g) => write!(f, "g{:02x}", g),
        }
    }
}

