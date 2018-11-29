use super::handle::Handle;
use super::opcode::{two_op, var_op, zero_op};
use super::opcode::{
    ZOperand, ZOperandType, EXTENDED_OPCODE_SENTINEL, OPCODE_TYPE_MASK, SHORT_OPCODE_TYPE_MASK,
    VAR_OPCODE_TYPE_MASK,
};
use super::result::Result;
use super::traits::{Header, Memory, Stack, Variables, PC};
use super::variables::ZVariables;
use super::version::ZVersion;

pub struct ZProcessor<H, M, P, S, V>
where
    H: Header,
    M: Memory,
    P: PC,
    S: Stack,
    V: Variables,
{
    pub memory: Handle<M>,
    pub header: H,
    pub pc: P,
    pub stack: Handle<S>,
    pub variables: V,
}

impl<H, M, P, S, V> ZProcessor<H, M, P, S, V>
where
    H: Header,
    M: Memory,
    P: PC,
    S: Stack,
    V: Variables,
{
    pub fn new(
        memory: Handle<M>,
        header: H,
        pc: P,
        stack: Handle<S>,
        variables: V,
    ) -> ZProcessor<H, M, P, S, V> {
        ZProcessor {
            memory,
            header,
            pc,
            stack,
            variables,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let cont = self.execute_opcode()?;
            if !cont {
                return Ok(());
            }
        }
    }

    // Result indicates whether or not we should continue.
    pub fn execute_opcode(&mut self) -> Result<bool> {
        let byte = self.pc.next_byte();
        if byte == EXTENDED_OPCODE_SENTINEL && self.header.version_number() >= ZVersion::V5 {
            self.execute_extended_opcode(byte)
        } else {
            // The top two bits indicate the opcode type.
            match byte & OPCODE_TYPE_MASK {
                SHORT_OPCODE_TYPE_MASK => self.execute_short_opcode(byte),
                VAR_OPCODE_TYPE_MASK => self.execute_var_opcode(byte),
                _ => self.execute_long_opcode(byte),
            }
        }
    }

    fn execute_extended_opcode(&mut self, byte: u8) -> Result<bool> {
        self.unimplemented("extended", byte)
    }

    fn execute_short_opcode(&mut self, byte: u8) -> Result<bool> {
        // For short opcodes, the low 4 bits contains the opcode.
        // Bits 4 & 5 contain the opcode type. (Omitted indicates no opcode, otherwise 1 opcode.)
        let opcode = byte & 0b1111;
        let optype = (byte & 0b0011_0000) >> 4;
        let operand = ZOperand::read_operand(&mut self.pc, optype.into());

        if let ZOperand::Omitted = operand {
            match opcode {
                11 => call_null(zero_op::o_187_new_line()),
                _ => self.unimplemented("0op", opcode),
            }
        } else {
            match opcode {
                _ => self.unimplemented("1op", opcode),
            }
        }
    }

    fn execute_var_opcode(&mut self, byte: u8) -> Result<bool> {
        // For var opcodes, the low 5 bits contain the opcode.
        let opcode = byte & 0b11111;

        // The 4 opcode types are encoded in the next byte.
        let optypes = self.pc.next_byte();

        let mut operands = <[ZOperand; 4]>::default();
        for idx in 0..4 {
            let optype = optypes >> ((3 - idx) * 2);
            let operand = ZOperand::read_operand(&mut self.pc, optype.into());
            match operand {
                ZOperand::Omitted => break,
                o => operands[idx] = o,
            }
        }

        match opcode {
            0 => call_null(var_op::o_224_call(
                &mut self.pc,
                &self.stack,
                &mut self.variables,
                &self.header.version_number(),
                operands,
            )),
            1 => call_null(var_op::o_225_storew(
                &self.memory,
                &mut self.variables,
                operands,
            )),
            3 => call_null(var_op::o_227_put_prop(operands)),
            _ => panic!("Unimplemented var opcode: {}", opcode),
        }
    }

    fn execute_long_opcode(&mut self, byte: u8) -> Result<bool> {
        let opcode = byte & 0b11111;
        let mut operands = <[ZOperand; 2]>::default();

        // Long opcodes use their own optype encoding. 0 = Small, 1 = Variable.
        //
        // Bit 6 encodes type of first operand.
        operands[0] = if byte & 0b0100_0000 == 0 {
            ZOperand::read_operand(&mut self.pc, ZOperandType::SmallConstantType)
        } else {
            ZOperand::read_operand(&mut self.pc, ZOperandType::VariableType)
        };

        // Bit 5 encodes type of second operand.
        operands[1] = if byte & 0b0010_0000 == 0 {
            ZOperand::read_operand(&mut self.pc, ZOperandType::SmallConstantType)
        } else {
            ZOperand::read_operand(&mut self.pc, ZOperandType::VariableType)
        };

        match opcode {
            0x01 => call_null(two_op::o_1_je(&mut self.pc, &mut self.variables, operands)),
            0x0a => call_null(two_op::o_10_test_attr(&mut self.pc, operands)),
            0x0d => call_null(two_op::o_13_store(&mut self.variables, operands)),
            0x14 => call_null(two_op::o_20_add(
                &mut self.pc,
                &mut self.variables,
                operands,
            )),
            0x15 => call_null(two_op::o_21_sub(
                &mut self.pc,
                &mut self.variables,
                operands,
            )),
            _ => self.unimplemented("long", opcode),
        }
    }

    fn unimplemented(&self, msg: &str, byte: u8) -> Result<bool> {
        panic!("Unimplemented {} opcode: {}", msg, byte);
    }
}

fn call_null(_n: ()) -> Result<bool> {
    Ok(true)
}

#[cfg(test)]
mod test {}
