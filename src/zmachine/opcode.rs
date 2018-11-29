use std::fmt;

use log::{debug, warn};

use super::addressing::{ByteAddress, PackedAddress};
use super::handle::Handle;
use super::result::Result;
use super::traits::{Memory, Stack, Variables, PC};
use super::version::ZVersion;

// Each (non-extended) opcode indicates its type (Short, Long, Var) with the top two bits.
pub const OPCODE_TYPE_MASK: u8 = 0b1100_0000;
pub const SHORT_OPCODE_TYPE_MASK: u8 = 0b1000_0000;
pub const VAR_OPCODE_TYPE_MASK: u8 = 0b1100_0000;

// In V5+, this opcode byte indicates that the second byte is an extended opcode.
pub const EXTENDED_OPCODE_SENTINEL: u8 = 0xbe;

// This is the only way that I can find to use these values as both constants in a 'match'
// and enum values.
const LARGE_CONSTANT_TYPE_BITS: u8 = 0b00;
const SMALL_CONSTANT_TYPE_BITS: u8 = 0b01;
const VARIABLE_TYPE_BITS: u8 = 0b10;
const OMITTED_TYPE_BITS: u8 = 0b11;

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

    fn maybe_value<V>(&self, variables: &mut V) -> Option<u16>
    where
        V: Variables,
    {
        match *self {
            ZOperand::Omitted => Option::None,
            _ => Option::Some(self.value(variables)),
        }
    }

    fn value<V>(&self, variables: &mut V) -> u16
    where
        V: Variables,
    {
        match *self {
            ZOperand::LargeConstant(val) => val,
            ZOperand::SmallConstant(val) => u16::from(val),
            ZOperand::Var(var) => variables.read_variable(var),
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

pub const MAX_LOCAL: u8 = 0x0e;
pub const MAX_GLOBAL: u8 = 0xef;

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
    pub fn o_187_new_line() {
        // TODO: This is not acceptible in a world with multiple output streams.
        println!("\n");
        debug!("new_line                        XXX");
    }
}

pub mod one_op {}

fn interpret_offset_byte<P>(byte: u8, pc: &mut P) -> i16
where
    P: PC,
{
    // TODO: move all of the pc manipulation here so that it can be called from all branches.
    if byte & 0b0100_0000 != 0 {
        // One byte only.
        i16::from(byte & 0b0011_1111)
    } else {
        let second_byte = pc.next_byte();
        let mut offset: u16 = ((byte as u16 & 0b0011_1111) << 8) + second_byte as u16;
        // Check for a negative 14-bit value, and sign extend to 16-bit if necessary.
        if offset & 0b0010_0000_0000_0000 != 0 {
            offset |= 0b1100_0000_0000_0000;
        }

        offset as i16
    }
}

fn branch<P, F>(byte: u8, pc: &mut P, tst: F)
where
    F: FnOnce(i16, bool) -> bool,
    P: PC,
{
    let branch_on_truth = !((byte & 0b1000_0000) == 0);
    let offset = interpret_offset_byte(byte, pc);

    let truth = tst(offset, branch_on_truth);

    if branch_on_truth == truth {
        // Branch!
        match offset {
            0 => panic!("unimplemented: ret false"),
            1 => panic!("unimplemented: ret true"),
            o => {
                pc.offset_pc((o - 2) as isize);
            }
        }
    }
}

pub mod two_op {
    use super::*;

    // ZSpec: 2OP:1 0x01 je a b ?(label)
    // UNTESTED
    pub fn o_1_je<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2])
    where
        P: PC,
        V: Variables,
    {
        let first_offset_byte = pc.next_byte();
        branch(first_offset_byte, pc, |offset, branch_on_truth| {
            debug!(
                "je          {} {} ?{}(x{:x})",
                operands[0],
                operands[1],
                if branch_on_truth { "" } else { "~" },
                offset
            );
            let first_val = operands[0].maybe_value(variables);
            let second_val = operands[1].maybe_value(variables);

            if first_val.is_some() && second_val.is_some() {
                first_val.unwrap() == second_val.unwrap()
            } else {
                false
            }
        });
    }

    // ZSpec: 2OP:10 0x0A test_attr object attribute ?(label)
    // UNTESTED
    pub fn o_10_test_attr<P>(pc: &mut P, operands: [ZOperand; 2])
    where
        P: PC,
    {
        let branch = pc.next_byte();
        debug!(
            "test_attr   {} {} ?{:b} XXX",
            operands[0], operands[1], branch
        );
    }

    // ZSpec: 2OP:13 0x0D store (variable) value
    pub fn o_13_store<V>(variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        V: Variables,
    {
        let variable = ZVariable::from(operands[0]);
        debug!("store       {} {}             XXX", variable, operands[1]);

        let value = operands[1].value(variables);
        variables.write_variable(variable, value)
    }

    // ZSpec: 2OP:20 0x14 add a b -> (result)
    pub fn o_20_add<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
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
            warn!("add {:x} + {:x} causes overflow.", lhs, rhs);
        }

        variables.write_variable(variable, result)
    }

    // ZSpec: TODO
    pub fn o_21_sub<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        let store = pc.next_byte();
        let variable = ZVariable::from(store);
        debug!(
            "sub         {} {} -> {}       XXX",
            operands[0], operands[1], variable
        );

        let lhs = operands[0].value(variables) as i16;
        let rhs = operands[1].value(variables) as i16;

        let (result, overflow) = lhs.overflowing_sub(rhs);
        if overflow {
            warn!("sub {:x} - {:x} causes overflow.", lhs, rhs);
        }

        variables.write_variable(variable, result as u16)
    }
}

pub mod var_op {
    use super::*;

    // ZSpec: VAR:224 0x00 V1 call routine ...up to 3 args... -> (result)
    // UNTESTED
    pub fn o_224_call<P, S, V>(
        pc: &mut P,
        stack: &Handle<S>,
        variables: &mut V,
        version: ZVersion,
        operands: [ZOperand; 4],
    ) where
        P: PC,
        S: Stack,
        V: Variables,
    {
        let store = pc.next_byte();

        let return_pc = pc.current_pc();

        // DO NOT SUBMIT. Make this a PackedAddress and DTRT.
        let packed = PackedAddress::new(operands[0].value(variables), version);

        pc.set_current_pc(packed.into());

        // Read function header.
        let num_locals = pc.next_byte();

        let mut local_values = [0u16; 15];
        if version < ZVersion::V5 {
            // On <V5, the function header also contains the starting values for the locals.
            for i in 0..num_locals {
                local_values[usize::from(i)] = pc.next_word();
            }
        }

        stack
            .borrow_mut()
            .push_frame(return_pc, num_locals, store.into(), &local_values);

        // TODO: do you ever push the arguments? I think you're not.
        // TODO: something is not right about the interaction between the routine header
        //       and the parameters. Write some test cases for this.

        // TODO: print operand[0] as a PackedAddress.
        debug!(
            "call        {} {} {} {} -> {}      XXX",
            packed, operands[1], operands[2], operands[3], store
        );
    }

    // ZSpec: VAR:225 0x01 storew array word-index value
    pub fn o_225_storew<M, V>(
        mem_h: &Handle<M>,
        variables: &mut V,
        operands: [ZOperand; 4],
    ) -> Result<()>
    where
        M: Memory,
        V: Variables,
    {
        debug!(
            "storew     {} {} {} {}             XXX",
            operands[0], operands[1], operands[2], operands[3]
        );

        let array = operands[0].value(variables);
        let word_index = operands[1].value(variables);
        let value = operands[2].value(variables);

        let ba = ByteAddress::from_raw(array).inc_by(2 * word_index);
        mem_h.borrow_mut().set_word(ba, value)
    }

    // ZSpec: VAR:227 0x03 put_prop object property value
    // UNTESTED
    pub fn o_227_put_prop(operands: [ZOperand; 4]) {
        debug!(
            "put_prop   {} {} {} {}             XXX",
            operands[0], operands[1], operands[2], operands[3]
        );
    }
}

#[cfg(test)]
mod test {
    use super::super::fixtures::*;
    use super::super::handle::new_handle;
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

        two_op::o_20_add(&mut pc, &mut variables, operands);

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

        two_op::o_20_add(&mut pc, &mut variables, operands);

        // Ensure that the pc advanced one byte.
        assert_eq!(9, pc.current_pc());
        assert_eq!(92, variables.variables[&ZVariable::Stack]);
    }

    #[test]
    fn test_store() {
        let mut variables = TestVariables::new();
        let operands: [ZOperand; 2] = [
            ZOperand::SmallConstant(0), // Stack
            ZOperand::LargeConstant(45),
        ];
        two_op::o_13_store(&mut variables, operands);

        assert_eq!(45, variables.variables[&ZVariable::Stack]);
    }

    #[test]
    fn test_storew() {
        let mut variables = TestVariables::new();
        let mut mem_h = new_handle(TestMemory::new(1000));
        let operands: [ZOperand; 4] = [
            ZOperand::SmallConstant(234),
            ZOperand::SmallConstant(5),
            ZOperand::LargeConstant(0xabcd),
            ZOperand::Omitted,
        ];

        var_op::o_225_storew(&mem_h, &mut variables, operands);

        assert_eq!(0xab, mem_h.borrow().bytes[244]);
        assert_eq!(0xcd, mem_h.borrow().bytes[245]);
    }

    use super::super::fixtures::TestPC;
    #[test]
    fn test_interpret_offset_byte() {
        let mut pc = TestPC::new(10, vec![0; 0]);
        assert_eq!(0b10_1010, interpret_offset_byte(0b0110_1010, &mut pc));

        let mut pc = TestPC::new(10, vec![0xab]);
        assert_eq!(0x0aab, interpret_offset_byte(0b0000_1010, &mut pc));

        let mut pc = TestPC::new(10, vec![0xab]);
        assert_eq!(
            0b1110_1010_1010_1011u32 as i16,
            interpret_offset_byte(0b0010_1010, &mut pc)
        );
    }

}
