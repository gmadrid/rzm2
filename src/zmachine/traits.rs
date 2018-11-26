use super::opcode::ZVariable;

trait Header {
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn next_byte(&mut self) -> u8;
    fn next_word(&mut self) -> u16;
}

trait Memory {
}

trait Stack {

}

pub trait Variables {
    fn read_variable(&self, var: &ZVariable) -> u16;
    fn write_variable(&mut self, var: &ZVariable, val: u16);
}
