use std::fmt;

use super::{PC, Result, Variables, ZErr, ZVariable};

#[derive(Clone, Copy, Debug)]
pub enum ZOperandType {
    LargeConstantType,
    SmallConstantType,
    VariableType,
    OmittedType,
}

impl From<u8> for ZOperandType {
    fn from(byte: u8) -> ZOperandType {
        // from must never fail, so it ignores the top bits.
        match byte & 0b11 {
            LARGE_CONSTANT_TYPE_BITS => ZOperandType::LargeConstantType,
            SMALL_CONSTANT_TYPE_BITS => ZOperandType::SmallConstantType,
            VARIABLE_TYPE_BITS => ZOperandType::VariableType,
            OMITTED_TYPE_BITS => ZOperandType::OmittedType,
            _ => panic!("This can't happen?"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ZOperand {
    LargeConstant(u16),
    SmallConstant(u8),
    Var(ZVariable),
    Omitted,
}

impl ZOperand {
    pub fn read_operand<P>(pc: &mut P, otype: ZOperandType) -> ZOperand
        where
            P: PC,
    {
        match otype {
            ZOperandType::LargeConstantType => {
                // Large constant
                let lc = pc.next_word();
                ZOperand::LargeConstant(lc)
            }
            ZOperandType::SmallConstantType => {
                // Small constant
                let sc = pc.next_byte();
                ZOperand::SmallConstant(sc)
            }
            ZOperandType::VariableType => {
                // Variable
                let var = pc.next_byte();
                ZOperand::Var(var.into())
            }
            // Omitted
            ZOperandType::OmittedType => ZOperand::Omitted,
        }
    }

    pub fn value<V>(&self, variables: &mut V) -> Result<u16>
        where
            V: Variables,
    {
        match *self {
            ZOperand::LargeConstant(val) => Ok(val),
            ZOperand::SmallConstant(val) => Ok(u16::from(val)),
            ZOperand::Var(var) => variables.read_variable(var),
            ZOperand::Omitted => Err(ZErr::MissingOperand),
        }
    }
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
            Var(v) => write!(f, "{}", v),
            Omitted => write!(f, "_"),
        }
    }
}
