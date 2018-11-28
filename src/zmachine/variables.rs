use super::handle::Handle;
use super::opcode::ZVariable;
use super::traits::{Memory, Stack, Variables};

pub struct ZVariables<M, S>
where
    M: Memory,
    S: Stack,
{
    mem_h: Handle<M>,
    stack_h: Handle<S>,
}

impl<M, S> ZVariables<M, S>
where
    M: Memory,
    S: Stack,
{
    pub fn new(mem_h: Handle<M>, stack_h: Handle<S>) -> ZVariables<M, S> {
        ZVariables { mem_h, stack_h }
    }

    fn pop_stack(&self) -> u16 {
        self.stack_h.borrow_mut().pop_word()
    }

    fn read_local(&self, l: u8) -> u16 {
        self.stack_h.borrow().read_local(l)
    }

    fn read_global(&self, g: u8) -> u16 {
        // XXX
        panic!("unimplemented");
    }
}

impl<M, S> Variables for ZVariables<M, S> where M: Memory, S: Stack {
    fn read_variable(&self, var: ZVariable) -> u16 {
        use self::ZVariable::*;
        match var {
            Stack => self.pop_stack(),
            Local(l) => self.read_local(l),
            Global(g) => self.read_global(g),
        }
    }

    fn write_variable(&mut self, var: ZVariable, val: u16) {
        panic!("unimplemented");
    }
}
