use super::opcode::ZVariable;
use super::version::ZVersion;

pub trait Header {
    fn version_number(&self) -> ZVersion;
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn next_byte(&mut self) -> u8;
    fn next_word(&mut self) -> u16;
}

pub trait Memory {}

pub trait Stack {}

pub trait Variables {
    fn read_variable(&self, var: &ZVariable) -> u16;
    fn write_variable(&mut self, var: &ZVariable, val: u16);
}
