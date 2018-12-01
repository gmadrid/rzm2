// UNREVIEWED

use super::addressing::ByteAddress;
use super::handle::Handle;
use super::opcode::{self, ZVariable};
use super::result::{Result, ZErr};
use super::traits::{Memory, Stack, Variables};

pub struct ZVariables<M, S>
where
    M: Memory,
    S: Stack,
{
    mem_h: Handle<M>,
    stack_h: Handle<S>,

    global_location: ByteAddress,
}

impl<M, S> ZVariables<M, S>
where
    M: Memory,
    S: Stack,
{
    pub fn new(
        global_location: ByteAddress,
        mem_h: Handle<M>,
        stack_h: Handle<S>,
    ) -> ZVariables<M, S> {
        ZVariables {
            mem_h,
            stack_h,
            global_location,
        }
    }

    fn pop_stack(&self) -> Result<u16> {
        Ok(self.stack_h.borrow_mut().pop_word())
    }

    fn push_stack(&self, word: u16) {
        self.stack_h.borrow_mut().push_word(word);
    }

    fn check_local_range(&self, l: u8) -> Result<()> {
        match l {
            0...opcode::MAX_LOCAL => Ok(()),
            _ => Err(ZErr::BadVariableIndex("local", l)),
        }
    }

    fn read_local(&self, l: u8) -> Result<u16> {
        self.check_local_range(l)?;
        Ok(self.stack_h.borrow().read_local(l))
    }

    fn write_local(&self, l: u8, word: u16) -> Result<()> {
        self.check_local_range(l)?;
        self.stack_h.borrow_mut().write_local(l, word);
        Ok(())
    }

    fn check_global_range(&self, g: u8) -> Result<()> {
        match g {
            0...opcode::MAX_GLOBAL => Ok(()),
            _ => Err(ZErr::BadVariableIndex("global", g)),
        }
    }

    fn read_global(&self, g: u8) -> Result<u16> {
        self.check_global_range(g)?;
        let offset = self.global_location.inc_by(2 * u16::from(g));
        Ok(self.mem_h.borrow().read_word(offset))
    }

    fn write_global(&self, g: u8, word: u16) -> Result<()> {
        self.check_global_range(g)?;
        let offset = self.global_location.inc_by(2 * u16::from(g));
        self.mem_h.borrow_mut().write_word(offset, word)
    }
}

impl<M, S> Variables for ZVariables<M, S>
where
    M: Memory,
    S: Stack,
{
    fn read_variable(&mut self, var: ZVariable) -> Result<u16> {
        use self::ZVariable::*;
        match var {
            Stack => self.pop_stack(),
            Local(l) => self.read_local(l),
            Global(g) => self.read_global(g),
        }
    }

    fn write_variable(&mut self, var: ZVariable, val: u16) -> Result<()> {
        use self::ZVariable::*;
        match var {
            Stack => {
                self.push_stack(val);
                Ok(())
            }
            Local(l) => {
                self.write_local(l, val)
            }
            Global(g) => self.write_global(g, val),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use zmachine::fixtures::{TestMemory, TestStack};
    use zmachine::handle::new_handle;

    fn make_test_setup() -> ZVariables<TestMemory, TestStack> {
        ZVariables::new(
            ByteAddress::from_raw(4),
            new_handle(TestMemory::new(0x400)),
            new_handle(TestStack::new(0x400)),
        )
    }

    #[test]
    fn test_variables_with_stack() {
        let mut variables = make_test_setup();

        variables.write_variable(ZVariable::Stack, 0x3579).unwrap();
        variables.write_variable(ZVariable::Stack, 0x4677).unwrap();
        variables.write_variable(ZVariable::Stack, 0xabcd).unwrap();

        assert_eq!(0xabcd, variables.read_variable(ZVariable::Stack).unwrap());
        assert_eq!(0x4677, variables.read_variable(ZVariable::Stack).unwrap());
        assert_eq!(0x3579, variables.read_variable(ZVariable::Stack).unwrap());
    }

    #[test]
    fn test_variables_with_locals() {
        let mut variables = make_test_setup();

        variables
            .write_variable(ZVariable::Local(3), 0x3579)
            .unwrap();
        variables
            .write_variable(ZVariable::Local(5), 0x4677)
            .unwrap();
        variables
            .write_variable(ZVariable::Local(7), 0xabcd)
            .unwrap();

        assert_eq!(0x3579, variables.read_variable(ZVariable::Local(3)).unwrap());
        assert_eq!(0x4677, variables.read_variable(ZVariable::Local(5)).unwrap());
        assert_eq!(0x3579, variables.read_variable(ZVariable::Local(3)).unwrap());
        assert_eq!(0xabcd, variables.read_variable(ZVariable::Local(7)).unwrap());
        assert_eq!(0x3579, variables.read_variable(ZVariable::Local(3)).unwrap());
    }

    #[test]
    fn test_variables_with_globals() {
        let mut variables = make_test_setup();

        variables
            .write_variable(ZVariable::Global(3), 0x3579)
            .unwrap();
        variables
            .write_variable(ZVariable::Global(5), 0x4677)
            .unwrap();
        variables
            .write_variable(ZVariable::Global(7), 0xabcd)
            .unwrap();

        assert_eq!(0x3579, variables.read_variable(ZVariable::Global(3)).unwrap());
        assert_eq!(0x4677, variables.read_variable(ZVariable::Global(5)).unwrap());
        assert_eq!(0x3579, variables.read_variable(ZVariable::Global(3)).unwrap());
        assert_eq!(0xabcd, variables.read_variable(ZVariable::Global(7)).unwrap());
        assert_eq!(0x3579, variables.read_variable(ZVariable::Global(3)).unwrap());
    }
}
