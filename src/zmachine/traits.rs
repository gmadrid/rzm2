use super::opcode::ZVariable;
use super::version::ZVersion;

pub trait Header {
    fn version_number(&self) -> ZVersion;
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn next_byte(&mut self) -> u8;

    fn next_word(&mut self) -> u16 {
        let high_byte = self.next_byte();
        let low_byte = self.next_byte();
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

pub trait Memory {}

pub trait Stack {
    fn pop_word(&mut self) -> u16;
}

pub trait Variables {
    fn read_variable(&self, var: &ZVariable) -> u16;
    fn write_variable(&mut self, var: &ZVariable, val: u16);
}
