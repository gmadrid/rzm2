#[derive(Debug)]
pub enum ZOperand {
    LargeConstant(u16),
    SmallConstant(u8),
    Variable(u8),
    Omitted,
}

impl Default for ZOperand {
    fn default() -> ZOperand {
        ZOperand::Omitted
    }
}

impl std::fmt::Display for ZOperand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use self::ZOperand::*;
        match *self {
            LargeConstant(c) => write!(f, "#{:04x}", c),
            SmallConstant(c) => write!(f, "#{:02x}", c),
            Variable(v) => {
                write!(f, "V:{:x}", v)
            },
            Omitted => write!(f, "_"),
        }
    }
}
