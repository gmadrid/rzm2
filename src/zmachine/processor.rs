use std::io::Read;

use super::addressing::PC;
use super::handle::Handle;
use super::header::ZHeader;
use super::memory::ZMemory;
use super::opcode::ZOperand;
use super::result::Result;
use super::stack::ZStack;
use super::version::ZVersion;

pub struct ZProcessor {
    pub story_h: Handle<ZMemory>,
    pub header: ZHeader,
    pub pc: PC,
    pub stack: ZStack,
}

impl ZProcessor {
    pub fn new<T: Read>(rdr: &mut T) -> Result<ZProcessor> {
        // TODO: error handling. get rid of unwraps.
        let (story_h, header) = ZMemory::new(rdr)?;
        // TODO: For V6, you will need to treat the start_pc as a PackedAddress.
        let pc = PC::new(&story_h, header.start_pc(), header.version_number());
        let stack = ZStack::new();

        Ok(ZProcessor {
            story_h,
            header,
            pc,
            stack,
        })
    }

    pub fn run(&mut self) {
        loop {
            self.execute_opcode();
        }
    }

    pub fn execute_opcode(&mut self) -> Result<()> {
        let byte = self.pc.next_byte();
        if byte == 0xbe && self.header.version_number() >= ZVersion::V5 {
            self.execute_extended_opcode(byte)
        } else {
            match byte & 0b11000000 {
                0b10000000 => self.execute_short_opcode(byte),
                0b11000000 => self.execute_var_opcode(byte),
                _ => self.execute_long_opcode(byte),
            }
        }
    }

    fn execute_extended_opcode(&mut self, byte: u8) -> Result<()> {
        self.unimplemented("extended", byte)
    }

    fn execute_short_opcode(&mut self, byte: u8) -> Result<()> {
        let opcode = byte & 0b1111;
        let optype = (byte & 0b00110000) >> 4;
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
                ZOperand::Variable(var)
            }
            // Omitted
            0b11 => ZOperand::Omitted,
            _ => panic!("This can't happen."),
        }
    }

    fn execute_long_opcode(&mut self, byte: u8) -> Result<()> {
        let opcode = byte & 0b11111;
        let mut operands = <[ZOperand; 2]>::default();

        operands[0] = if byte & 0b01000000 == 0 {
            self.read_operand_of_type(0b01)
        } else {
            self.read_operand_of_type(0b10)
        };

        operands[1] = if byte & 0b00100000 == 0 {
            self.read_operand_of_type(0b01)
        } else {
            self.read_operand_of_type(0b10)
        };

        match opcode {
            0x0a => self.twoop_10_test_attr(operands),
            0x0d => self.twoop_13_store(operands),
            0x14 => self.twoop_20_add(operands),
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
        println!("store       ({}) {}           XXX", operands[0], operands[1]);
        Ok(())
    }

    fn twoop_20_add(&mut self, operands: [ZOperand; 2]) -> Result<()> {
        let store = self.pc.next_byte();
        println!("add         {} {} -> {}       XXX", operands[0], operands[1], store);
        Ok(())
    }

    fn noops_187_new_line(&mut self) -> Result<()> {
        println!("new_line                        XXX");
        Ok(())
    }

    fn var_224_call(&mut self, operands: [ZOperand; 4]) -> Result<()> {
        let store = self.pc.next_byte();
        println!(
            "call        {} {} {} {}       XXX",
            operands[0], operands[1], operands[2], operands[3]
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
