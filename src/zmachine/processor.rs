use super::handle::Handle;
use super::opcode::{one_op, two_op, var_op, zero_op};
use super::opcode::{ZOperand, ZOperandType};
use super::opcode::{
    EXTENDED_OPCODE_SENTINEL, OPCODE_TYPE_MASK, SHORT_OPCODE_TYPE_MASK, VAR_OPCODE_TYPE_MASK,
};
use super::result::{Result, ToTrue, ZErr};
use super::traits::{Header, Memory, Stack, Variables, PC};
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
        while self.execute_opcode()? {}
        return Ok(());
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
                0x00 => zero_op::o_176_rtrue(&mut self.pc, &self.stack, &mut self.variables).to_true(),
                0x02 => {
                    zero_op::o_178_print(&self.memory, &mut self.pc, self.header.abbrev_location())
                        .to_true()
                }
                0x0b => call_null(zero_op::o_187_new_line()),
                _ => self.unimplemented("0op", opcode),
            }
        } else {
            match opcode {
                0x00 => one_op::o_128_jz(&mut self.pc, &mut self.variables, operand).to_true(),
                0x0b => one_op::o_139_ret(&mut self.pc, &self.stack, &mut self.variables, operand)
                    .to_true(),
                0x0c => one_op::o_140_jump(&mut self.pc, &mut self.variables, operand).to_true(),
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

        if byte & 0b0010_0000 == 0 {
            self.match_long_opcode(opcode, [operands[0], operands[1]])
        } else {
        match opcode {
            0 => var_op::o_224_call(
                &mut self.pc,
                &self.stack,
                &mut self.variables,
                self.header.version_number(),
                operands,
            ).to_true(),
            1 => var_op::o_225_storew(&self.memory, &mut self.variables, operands).to_true(),
            3 => call_null(var_op::o_227_put_prop(operands)),
            5 => var_op::o_229_print_char(&mut self.variables, operands).to_true(),
            6 => var_op::o_230_print_num(&mut self.variables, operands).to_true(),
            _ => panic!("Unimplemented var opcode: {}", opcode),
        }
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

        self.match_long_opcode(opcode, operands)
    }

    fn match_long_opcode(&mut self, opcode: u8, operands: [ZOperand; 2]) -> Result<bool> {
        match opcode {
            0x01 => two_op::o_1_je(&mut self.pc, &mut self.variables, operands).to_true(),
            0x05 => two_op::o_5_inc_chk(&mut self.pc, &mut self.variables, operands).to_true(),
            0x09 => two_op::o_9_and(&mut self.pc, &mut self.variables, operands).to_true(),
            0x0a => call_null(two_op::o_10_test_attr(&mut self.pc, operands)),
            0x0d => two_op::o_13_store(&mut self.variables, operands).to_true(),
            0x0f => two_op::o_15_loadw(
                &mut self.memory,
                &mut self.pc,
                &mut self.variables,
                operands,
            ).to_true(),
            0x10 => two_op::o_16_loadb(
                &mut self.memory,
                &mut self.pc,
                &mut self.variables,
                operands,
            ).to_true(),
            0x14 => two_op::o_20_add(&mut self.pc, &mut self.variables, operands).to_true(),
            0x15 => two_op::o_21_sub(&mut self.pc, &mut self.variables, operands).to_true(),
            _ => self.unimplemented("long", opcode),
        }
    }

    fn unimplemented(&self, msg: &'static str, byte: u8) -> Result<bool> {
        Err(ZErr::UnknownOpcode(msg, u16::from(byte)))
    }
}

fn call_null(_n: ()) -> Result<bool> {
    Ok(true)
}

#[cfg(test)]
mod test {
    // One day, you might want to figure out how to test this.
}
