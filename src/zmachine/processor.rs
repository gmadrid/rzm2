use std::io::Read;

use super::addressing::ZPC;
use super::handle::Handle;
use super::header::ZHeader;
use super::memory::ZMemory;
use super::opcode::{self, ZOperand, ZVariable};
use super::result::Result;
use super::stack::ZStack;
use super::traits::PC;
use super::version::ZVersion;

pub struct ZProcessor<P>
where
    P: PC,
{
    pub story_h: Handle<ZMemory>,
    pub header: ZHeader,
    pub pc: P,
    pub stack: ZStack,
}

pub fn new_processor_from_rdr<T: Read>(rdr: &mut T) -> Result<ZProcessor<ZPC>> {
    // TODO: error handling. get rid of unwraps.
    let (story_h, header) = ZMemory::new(rdr)?;
    // TODO: For V6, you will need to treat the start_pc as a PackedAddress.
    let pc = ZPC::new(&story_h, header.start_pc(), header.version_number());
    let stack = ZStack::new();

    Ok(ZProcessor::new(story_h, header, pc, stack))
}

impl<P> ZProcessor<P>
where
    P: PC,
{
    pub fn new(memory_h: Handle<ZMemory>, header: ZHeader, pc: P, stack: ZStack) -> ZProcessor<P> {
        ZProcessor {
            story_h: memory_h,
            header,
            pc,
            stack,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let result = self.execute_opcode();
            if let Err(_) = result {
                return result;
            }
        }
    }

    pub fn execute_opcode(&mut self) -> Result<()> {
        let byte = self.pc.next_byte();
        if byte == 0xbe && self.header.version_number() >= ZVersion::V5 {
            self.execute_extended_opcode(byte)
        } else {
            match byte & 0b1100_0000 {
                0b1000_0000 => self.execute_short_opcode(byte),
                0b1100_0000 => self.execute_var_opcode(byte),
                _ => self.execute_long_opcode(byte),
            }
        }
    }

    fn execute_extended_opcode(&mut self, byte: u8) -> Result<()> {
        self.unimplemented("extended", byte)
    }

    fn execute_short_opcode(&mut self, byte: u8) -> Result<()> {
        let opcode = byte & 0b1111;
        let optype = (byte & 0b0011_0000) >> 4;
        let operand = self.read_operand_of_type(optype);

        if let ZOperand::Omitted = operand {
            match opcode {
                11 => self.noops_187_new_line(),
                _ => self.unimplemented("0op", opcode),
            }
        } else {
            self.unimplemented("1op", opcode)
        }
    }

    fn execute_var_opcode(&mut self, byte: u8) -> Result<()> {
        let opcode = byte & 0b11111;
        let optypes = self.pc.next_byte();

        let mut operands = <[ZOperand; 4]>::default();
        for idx in 0..4 {
            let operand = self.read_operand_of_type(optypes >> ((3 - idx) * 2));
            match operand {
                ZOperand::Omitted => break,
                o => operands[idx] = o,
            }
        }

        match opcode {
            0 => self.var_224_call(operands),
            1 => self.var_225_storew(operands),
            3 => self.var_227_put_prop(operands),
            _ => panic!("Unimplemented var opcode: {}", opcode),
        }
    }

    fn read_operand_of_type(&mut self, byte: u8) -> ZOperand {
        match byte & 0b11 {
            0b00 => {
                // Large constant
                let lc = self.pc.next_word();
                ZOperand::LargeConstant(lc)
            }
            0b01 => {
                // Small constant
                let sc = self.pc.next_byte();
                ZOperand::SmallConstant(sc)
            }
            0b10 => {
                // Variable
                let var = self.pc.next_byte();
                ZOperand::Var(var.into())
            }
            // Omitted
            0b11 => ZOperand::Omitted,
            _ => panic!("This can't happen."),
        }
    }

    fn execute_long_opcode(&mut self, byte: u8) -> Result<()> {
        let opcode = byte & 0b11111;
        let mut operands = <[ZOperand; 2]>::default();

        // Bit 6 encodes type of first operand.
        operands[0] = if byte & 0b0100_0000 == 0 {
            self.read_operand_of_type(0b01)
        } else {
            self.read_operand_of_type(0b10)
        };

        // Bit 5 encodes type of second operand.
        operands[1] = if byte & 0b0010_0000 == 0 {
            self.read_operand_of_type(0b01)
        } else {
            self.read_operand_of_type(0b10)
        };

        match opcode {
            0x0a => self.twoop_10_test_attr(operands),
            0x0d => self.twoop_13_store(operands),
            0x14 => opcode::two_op::o_20_add(&mut self.pc, &mut self, operands),
            _ => self.unimplemented("long", opcode),
        }
    }

    fn unimplemented(&self, msg: &str, byte: u8) -> Result<()> {
        panic!("Unimplemented {} opcode: {}", msg, byte);
    }

    fn twoop_10_test_attr(&mut self, operands: [ZOperand; 2]) -> Result<()> {
        let branch = self.pc.next_byte();
        println!(
            "test_attr   {} {} ?{:b} XXX",
            operands[0], operands[1], branch
        );
        Ok(())
    }

    fn twoop_13_store(&mut self, operands: [ZOperand; 2]) -> Result<()> {
        // 2OP:13 0x0D store (variable) value
        let variable = ZVariable::from(operands[0]);
        println!("store       ({}) {}           XXX", variable, operands[1]);
        Ok(())
    }

    fn noops_187_new_line(&mut self) -> Result<()> {
        println!("new_line                        XXX");
        Ok(())
    }

    fn var_224_call(&mut self, operands: [ZOperand; 4]) -> Result<()> {
        // 1) Save away old PC. It is the return value.
        // 2) Set PC to new value.
        // 3) Read num vars/num locals from new location.
        // 4) Push new frame onto stack.
        //    - return Offset
        //    - Old frame ptr
        //    - locals init
        //    - leave space for locals
        let store = self.pc.next_byte();

        //        let next_pc = self.pc.current_pc();
        //        let pa = self.header.version_number().make_packed_address(val);

        println!(
            "call        {} {} {} {} -> {}      XXX",
            operands[0], operands[1], operands[2], operands[3], store
        );
        Ok(())
    }

    fn var_225_storew(&mut self, operands: [ZOperand; 4]) -> Result<()> {
        println!("XXX storew not done");
        Ok(())
    }

    fn var_227_put_prop(&mut self, operands: [ZOperand; 4]) -> Result<()> {
        println!("XXX put_prop not done");
        Ok(())
    }
}

#[cfg(test)]
mod test {}
