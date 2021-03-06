// UNREVIEWED

use std::fmt;

use log::{debug, warn};

use super::addressing::ByteAddress;
use super::handle::Handle;
use super::result::{Result, ZErr};
use super::traits::{Memory, Stack, Variables, PC};
use super::version::ZVersion;
use super::zscii::read_zstr_from_pc;

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

    fn value<V>(&self, variables: &mut V) -> Result<u16>
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
            ZOperand::LargeConstant(lc) => (lc as u8).into(),
            // TODO: XXX finish this.
            _ => unimplemented!("From<ZOperand> for ZVariable"),
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

pub fn return_value<P, S, V>(
    value: u16,
    pc: &mut P,
    stack: &Handle<S>,
    variables: &mut V,
) -> Result<()>
where
    P: PC,
    S: Stack,
    V: Variables,
{
    let return_pc = stack.borrow().return_pc();
    let return_variable = stack.borrow().return_variable();
    stack.borrow_mut().pop_frame()?;
    variables.write_variable(return_variable, value)?;
    pc.set_current_pc(return_pc);
    Ok(())
}

pub mod zero_op {
    use super::*;

    // ZSpec: 0OP:176 0x00 rtrue
    // UNTESTED
    pub fn o_176_rtrue<P, S, V>(pc: &mut P, stack: &Handle<S>, variables: &mut V) -> Result<()>
    where
        P: PC,
        S: Stack,
        V: Variables,
    {
        debug!("rtrue");
        return_value(1, pc, stack, variables)
    }

    // ZSpec: 0OP:177 0x01 rfalse
    // UNTESTED
    pub fn o_177_rfalse<P, S, V>(pc: &mut P, stack: &Handle<S>, variables: &mut V) -> Result<()>
    where
        P: PC,
        S: Stack,
        V: Variables,
    {
        debug!("rfalse");
        return_value(0, pc, stack, variables)
    }

    // ZSpec: 0OP:178 0x02 print (literal-string)
    // UNTESTED
    pub fn o_178_print<M, P>(
        memory: &Handle<M>,
        pc: &mut P,
        abbrev_offset: ByteAddress,
    ) -> Result<()>
    where
        M: Memory,
        P: PC,
    {
        // TODO: This is not acceptible in a world with multiple output streams.
        debug!("print");
        let zstr = read_zstr_from_pc(&memory, abbrev_offset, pc)?;
        print!("{}", zstr);
        Ok(())
    }

    // ZSpec: 0OP:187 0x0B new_line
    // UNTESTED
    pub fn o_187_new_line() {
        // TODO: This is not acceptible in a world with multiple output streams.
        println!("\n");
        debug!("new_line");
    }
}

pub mod one_op {
    use super::*;

    // ZSpec: 1OP:128 0x00 jz a ?(label)
    // UNTESTED
    pub fn o_128_jz<P, V>(pc: &mut P, variables: &mut V, operand: ZOperand) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        let first_offset_byte = pc.next_byte();
        branch(first_offset_byte, pc, |offset, branch_on_truth| {
            debug!(
                "jz         {} ?{}(x{:x})",
                operand,
                if branch_on_truth { "" } else { "~" },
                offset
            );

            // TODO: what if this is Omitted?
            Ok(operand.value(variables)? == 0)
        })
    }

    // ZSpec: 1OP:139 0x0b ret value
    // UNTESTED
    pub fn o_139_ret<P, S, V>(
        pc: &mut P,
        stack: &Handle<S>,
        variables: &mut V,
        operand: ZOperand,
    ) -> Result<()>
    where
        P: PC,
        S: Stack,
        V: Variables,
    {
        let result = operand.value(variables)?;
        debug!("ret         {}", operand);
        return_value(result, pc, stack, variables)
    }

    // ZSpec: 1OP:140 0x0c jump ?(label)
    // UNTESTED
    pub fn o_140_jump<P, V>(pc: &mut P, variables: &mut V, operand: ZOperand) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        debug!("jump       {}", operand);

        let offset = isize::from(operand.value(variables)? as i16) - 2;
        pc.offset_pc(offset);
        Ok(())
    }
}

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

fn branch<P, F>(byte: u8, pc: &mut P, tst: F) -> Result<()>
where
    F: FnOnce(i16, bool) -> Result<bool>,
    P: PC,
{
    // TODO: do all offset handling (and reading from PC in interpret_offset_byte.
    let branch_on_truth = !((byte & 0b1000_0000) == 0);
    let offset = interpret_offset_byte(byte, pc);

    let truth = tst(offset, branch_on_truth)?;

    if branch_on_truth == truth {
        // Branch!
        match offset {
            0 => unimplemented!("ret false"),
            1 => unimplemented!("ret true"),
            o => {
                pc.offset_pc((o - 2) as isize);
            }
        }
    }
    Ok(())
}

pub mod two_op {
    use super::*;

    // ZSpec: 2OP:1 0x01 je a b ?(label)
    // UNTESTED
    pub fn o_1_je<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
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
            let first_val = operands[0].value(variables);
            let second_val = operands[1].value(variables);

            // TODO: this needs to deal with the case when there are > 2 arguments. It's a real thing.

            if first_val.is_ok() && second_val.is_ok() {
                Ok(first_val.unwrap() == second_val.unwrap())
            } else {
                Ok(false)
            }
        })
    }

    // ZSpec: 2OP:5 0x05 inc_chk (variable) value ?(label)
    // UNTESTED
    pub fn o_5_inc_chk<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        let variable = ZVariable::from(operands[0].value(variables)? as u8);
        let first_offset_byte = pc.next_byte();
        branch(first_offset_byte, pc, |offset, branch_on_truth| {
            debug!(
                "inc_chk    {} {} ?{}({:x})",
                variable,
                operands[1],
                if branch_on_truth { "" } else { "~" },
                offset
            );

            let old_value = variables.read_variable(variable)?;
            let (result, overflow) = old_value.overflowing_add(1);
            if overflow {
                warn!("inc_chk    {} causes overflow.", variable);
            }
            variables.write_variable(variable, result)?;

            let test_value = operands[1].value(variables)?;
            Ok(result > test_value)
        })
    }

    // ZSpec: 2OP:9 0x09 and a b -> (result)
    // UNTESTED
    pub fn o_9_and<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        let store = ZVariable::from(pc.next_byte());

        let lhs = operands[0].value(variables)?;
        let rhs = operands[1].value(variables)?;

        debug!("and        {} {} -> {}", operands[0], operands[1], store);

        variables.write_variable(store, lhs & rhs)
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
        unimplemented!("test_attr")
    }

    // ZSpec: 2OP:13 0x0D store (variable) value
    pub fn o_13_store<V>(variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        V: Variables,
    {
        let variable = ZVariable::from(operands[0]);
        debug!("store       {} {}", variable, operands[1]);

        let value = operands[1].value(variables)?;
        variables.write_variable(variable, value)
    }

    // ZSpec: 2OP:15 0x0f loadw array word-index -> (result)
    // UNTESTED
    pub fn o_15_loadw<M, P, V>(
        memory: &Handle<M>,
        pc: &mut P,
        variables: &mut V,
        operands: [ZOperand; 2],
    ) -> Result<()>
    where
        M: Memory,
        P: PC,
        V: Variables,
    {
        let store = ZVariable::from(pc.next_byte());
        debug!("loadw      {} {} -> {}", operands[0], operands[1], store);

        let array = operands[0].value(variables)?;
        let word_index = operands[1].value(variables)?;

        let byte_address = ByteAddress::from_raw(array).inc_by(2 * word_index);
        let value = memory.borrow().read_word(byte_address);
        variables.write_variable(store, value)
    }

    // ZSpec: 2OP:16 0x10 loadb array byte-index -> (result)
    // UNTESTED
    pub fn o_16_loadb<M, P, V>(
        memory: &Handle<M>,
        pc: &mut P,
        variables: &mut V,
        operands: [ZOperand; 2],
    ) -> Result<()>
    where
        M: Memory,
        P: PC,
        V: Variables,
    {
        let store = ZVariable::from(pc.next_byte());
        debug!("loadb      {} {} -> {}", operands[0], operands[1], store);

        let array = operands[0].value(variables)?;
        let byte_index = operands[1].value(variables)?;

        let byte_address = ByteAddress::from_raw(array).inc_by(byte_index);
        let value = memory.borrow().read_byte(byte_address);
        variables.write_variable(store, u16::from(value))
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
            "add         {} {} -> {}",
            operands[0], operands[1], variable
        );

        let lhs = operands[0].value(variables)?;
        let rhs = operands[1].value(variables)?;

        let (result, overflow) = lhs.overflowing_add(rhs);
        if overflow {
            warn!("add {:x} + {:x} causes overflow.", lhs, rhs);
        }

        variables.write_variable(variable, result)
    }

    // ZSpec: TODO
    // UNTESTED
    pub fn o_21_sub<P, V>(pc: &mut P, variables: &mut V, operands: [ZOperand; 2]) -> Result<()>
    where
        P: PC,
        V: Variables,
    {
        let store = pc.next_byte();
        let variable = ZVariable::from(store);
        debug!(
            "sub         {} {} -> {}",
            operands[0], operands[1], variable
        );

        let lhs = operands[0].value(variables)? as i16;
        let rhs = operands[1].value(variables)? as i16;

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
    ) -> Result<()>
    where
        P: PC,
        S: Stack,
        V: Variables,
    {
        let store = pc.next_byte();

        let return_pc = pc.current_pc();

        let packed = version.make_packed_address(operands[0].value(variables)?);
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
            .push_frame(return_pc, num_locals, store.into(), &local_values)?;

        // TODO: do you ever push the arguments? I think you're not.
        // TODO: something is not right about the interaction between the routine header
        //       and the parameters. Write some test cases for this.

        // TODO: print operand[0] as a PackedAddress.
        debug!(
            "call        {} {} {} {} -> {}",
            packed, operands[1], operands[2], operands[3], store
        );
        Ok(())
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
            "storew      {} {} {} {}",
            operands[0], operands[1], operands[2], operands[3]
        );

        let array = operands[0].value(variables)?;
        let word_index = operands[1].value(variables)?;
        let value = operands[2].value(variables)?;

        let ba = ByteAddress::from_raw(array).inc_by(2 * word_index);
        mem_h.borrow_mut().write_word(ba, value)
    }

    // ZSpec: VAR:227 0x03 put_prop object property value
    // UNTESTED
    pub fn o_227_put_prop(operands: [ZOperand; 4]) {
        debug!(
            "put_prop   {} {} {} {}             XXX",
            operands[0], operands[1], operands[2], operands[3]
        );
        unimplemented!("put_prop")
    }

    // ZSpec: VAR:229 0x05 print_char output_character_code
    // UNTESTED
    pub fn o_229_print_char<V>(variables: &mut V, operands: [ZOperand; 4]) -> Result<()>
    where
        V: Variables,
    {
        debug!("print_char {}", operands[0]);
        // TODO: deal with the case where extra argements are passed.
        //       stuff will break if an extra SP arg is passed, but never popped.
        let ch = operands[0].value(variables)? as u8 as char;
        print!("{}", ch);
        Ok(())
    }

    // ZSpec: VAR:230 0x06 print_num value
    // UNTESTED
    pub fn o_230_print_num<V>(variables: &mut V, operands: [ZOperand; 4]) -> Result<()>
    where
        V: Variables,
    {
        debug!(
            "print_num  {} {} {} {}",
            operands[0], operands[1], operands[2], operands[3]
        );

        let num = operands[0].value(variables)?;
        print!("{}", (num as i16));

        Ok(())
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

    #[test]
    fn test_store() {
        let mut variables = TestVariables::new();
        let operands: [ZOperand; 2] = [
            ZOperand::SmallConstant(0), // Stack
            ZOperand::LargeConstant(45),
        ];
        two_op::o_13_store(&mut variables, operands).unwrap();

        assert_eq!(45, variables.variables[&ZVariable::Stack]);
    }

    #[test]
    fn test_storew() {
        let mut variables = TestVariables::new();
        let mem_h = new_handle(TestMemory::new(1000));
        let operands: [ZOperand; 4] = [
            ZOperand::SmallConstant(234),
            ZOperand::SmallConstant(5),
            ZOperand::LargeConstant(0xabcd),
            ZOperand::Omitted,
        ];

        var_op::o_225_storew(&mem_h, &mut variables, operands).unwrap();

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
