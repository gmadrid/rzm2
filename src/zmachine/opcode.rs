use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum ZOperand {
    LargeConstant(u16),
    SmallConstant(u8),
    Var(ZVariable),
    Omitted,
}

impl Default for ZOperand {
    fn default() -> ZOperand {
        ZOperand::Omitted
    }
}

impl fmt::Display for ZOperand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ZOperand::*;
        match *self {
            LargeConstant(c) => write!(f, "#{:04x}", c),
            SmallConstant(c) => write!(f, "#{:02x}", c),
            Var(v) => {
                write!(f, "{}", v)
            },
            Omitted => write!(f, "_"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ZVariable {
    Stack,
    Local(u8),  // 0..e
    Global(u8), // 0..ef
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

// This is mainly for "indirect" operands.
// panic! if value is out of range.
impl From<ZOperand> for ZVariable {
    fn from(operand: ZOperand) -> ZVariable {
        match operand {
            ZOperand::SmallConstant(c) => c.into(),
            // TODO: XXX finish this.
            _ => panic!("Not done yet. XXX"),
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
