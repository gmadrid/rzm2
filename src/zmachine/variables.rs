use super::opcode::ZVariable;
use super::traits::Variables;

pub struct ZVariables {}

impl Variables for ZVariables {
    fn read_variable(&self, var: ZVariable) -> u16 {
        panic!("unimplemented");
    }

    fn write_variable(&mut self, var: ZVariable, val: u16) {
        panic!("unimplemented");
    }
}
