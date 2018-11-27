use std::fmt;

use log::{debug, warn};

use super::result::Result;
use super::traits::{Memory, Variables, PC};

// Each (non-extended) opcode indicates its type (Short, Long, Var) with the top two bits.
pub const OPCODE_TYPE_MASK: u8 = 0b1100_0000;
pub const SHORT_OPCODE_TYPE_MASK: u8 = 0b1000_0000;
pub const VAR_OPCODE_TYPE_MASK: u8 = 0b1100_0000;

// In V5+, this opcode byte indicates that the second byte is an extended opcode.
pub const EXTENDED_OPCODE_SENTINEL: u8 = 0xbe;

// This is the only way that I can find to use these values as both constants in a 'match'
// and enum values.
const LargeConstantTypeConst: u8 = 0b00;
const SmallConstantTypeConst: u8 = 0b01;
const VariableTypeConst: u8 = 0b10;
const OmittedTypeConst: u8 = 0b11;

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
            LargeConstantTypeConst => ZOperandType::LargeConstantType,
            SmallConstantTypeConst => ZOperandType::SmallConstantType,
            VariableTypeConst => ZOperandType::VariableType,
            OmittedTypeConst => ZOperandType::OmittedType,
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

    fn value<V>(&self, variables: &mut V) -> u16
    where
        V: Variables,
    {
        match *self {
            ZOperand::LargeConstant(val) => val,
            ZOperand::SmallConstant(val) => u16::from(val),
            ZOperand::Var(var) => variables.read_variable(&var),
            ZOperand::Omitted => panic!("Cannot load value from an Omitted operand."),
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

pub mod zero_op {
    use super::*;

    // ZSpec: 0OP:187 0x0B new_line
    // UNTESTED
    pub fn o_187_new_line() -> Result<bool> {
        // TODO: This is not acceptible in a world with multiple output streams.
        println!("\n");
        debug!("new_line                        XXX");
        Ok(true)
    }
}

pub mod one_op {}

pub mod two_op {
    use super::*;

    // ZSpec: 2OP:10 0x0A test_attr object attribute ?(label)
    // UNTESTED
    pub fn o_10_test_attr<P>(pc: &mut P, operands: [ZOperand; 2]) -> Result<bool>
    where
        P: PC,
    {
        let branch = pc.next_byte();
        debug!(
            "test_attr   {} {} ?{:b} XXX",
            operands[0], operands[1], branch
        );
        Ok(true)
    }

    // ZSpec: 2OP:13 0x0D store (variable) value
    // UNTESTED
    pub fn o_13_store(operands: [ZOperand; 2]) -> Result<bool> {
        let variable = ZVariable::from(operands[0]);
        debug!("store       {} {}             XXX", variable, operands[1]);
        Ok(true)
    }

    // ZSpec: 2OP:20 0x14 add a b -> (result)
    pub fn o_20_add<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<bool>
    where
        P: PC,
        V: Variables,
    {
        let store = pc.next_byte();
        let variable = ZVariable::from(store);
        debug!(
            "add         {} {} -> {}       XXX",
            operands[0], operands[1], variable
        );

        let lhs = operands[0].value(variables);
        let rhs = operands[1].value(variables);

        let (result, overflow) = lhs.overflowing_add(rhs);
        if overflow {
            warn!("add {} + {} causes overflow.", lhs, rhs);
        }

        variables.write_variable(&variable, result);

        Ok(true)
    }

}

pub mod var_op {
    use super::*;

    // ZSpec: VAR:224 0x00 V1 call routine ...up to 3 args... -> (result)
    // UNTESTED
    pub fn o_224_call<P>(pc: &mut P, operands: [ZOperand; 4]) -> Result<bool>
    where
        P: PC,
    {
        // 1) Save away old PC. It is the return value.
        // 2) Set PC to new value.
        // 3) Read num vars/num locals from new location.
        // 4) Push new frame onto stack.
        //    - return Offset
        //    - Old frame ptr
        //    - locals init
        //    - leave space for locals
        let store = pc.next_byte();

        //        let next_pc = self.pc.current_pc();
        //        let pa = self.header.version_number().make_packed_address(val);

        debug!(
            "call        {} {} {} {} -> {}      XXX",
            operands[0], operands[1], operands[2], operands[3], store
        );
        Ok(true)
    }

    // ZSpec: VAR:225 1 storew array word-index value
    // UNTESTED
    pub fn o_225_storew(operands: [ZOperand; 4]) -> Result<bool> {
        debug!(
            "storew     {} {} {} {}             XXX",
            operands[0], operands[1], operands[2], operands[3]
        );
        Ok(true)
    }

    // ZSpec: VAR:227 3 put_prop object property value
    // UNTESTED
    pub fn o_227_put_prop(operands: [ZOperand; 4]) -> Result<bool> {
        debug!(
            "put_prop   {} {} {} {}             XXX",
            operands[0], operands[1], operands[2], operands[3]
        );
        Ok(true)
    }
}

#[cfg(test)]
mod test {
    use super::super::fixtures::*;
    use super::*;

    #[test]
    fn test_add() {
        let mut pc = TestPC::new(
            8,
            vec![
                0, // Stack
            ],
        );
        let mut variables = TestVariables::new();
        let operands: [ZOperand; 2] = [ZOperand::SmallConstant(3), ZOperand::LargeConstant(98)];

        two_op::o_20_add(&mut pc, &mut variables, operands).unwrap();

        // Ensure that the pc advanced one byte.
        assert_eq!(9, pc.current_pc());
        assert_eq!(101, variables.variables[&ZVariable::Stack]);
    }

    #[test]
    fn test_add_overflow() {
        let mut pc = TestPC::new(
            8,
            vec![
                0, // Stack
            ],
        );
        let mut variables = TestVariables::new();
        let operands: [ZOperand; 2] = [ZOperand::LargeConstant(65530), ZOperand::SmallConstant(98)];

        two_op::o_20_add(&mut pc, &mut variables, operands).unwrap();

        // Ensure that the pc advanced one byte.
        assert_eq!(9, pc.current_pc());
        assert_eq!(92, variables.variables[&ZVariable::Stack]);
    }
}
