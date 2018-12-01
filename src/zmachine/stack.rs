// UNREVIEWED

use super::constants;
use super::opcode::ZVariable;
use super::result::{Result, ZErr};
use super::traits::{bytes, Stack};

pub struct ZStack {
    stack: [u8; constants::STACK_SIZE],

    fp: usize, // index in the stack of the current frame.

    s0: usize, // The bottom of the current frame's stack.
    // (The first byte after the local variables.)
    sp: usize, // points to the next empty byte.
               // Initialized to s0.
}

// Each frame has the following fields.
//
//   fp: u16         - index on the stack of the previous frame.
//                     (The top frame has STACK_SIZE here.)
//   return_pc:  u32 - Next pc value after returning.
//   return_var: u8  - Encoded ZVariable for return value.
//   num_locals: u8  - Number of local variables on the stack. (0-14)
//   locals: u16     - One of these for each local, so up to 14.
//
// NOTE: we can probably save one byte/frame (and preserve word-alignment) by
//       only storing 24 bits of the return_pc. (No legal story will overflow 24-bits.)
//
impl ZStack {
    const SAVED_PC_OFFSET: usize = 0;
    const RETURN_PC_OFFSET: usize = 2;
    const RETURN_VAR_OFFSET: usize = 6;
    const NUM_LOCALS_OFFSET: usize = 7;
    const LOCAL_VAR_OFFSET: usize = 8;

    pub fn new() -> ZStack {
        let mut zs = ZStack {
            stack: [0; constants::STACK_SIZE],
            fp: 0,
            s0: 0,
            sp: 0,
        };

        // If this fails, it is programmer error.
        zs.init_new_stack().unwrap();

        zs.s0 = zs.sp;

        zs
    }

    //
    // Create a pseudo-frame for the base frame.
    //
    fn init_new_stack(&mut self) -> Result<()> {
        // There is not previous frame, so point to an illegal value.
        self.push_word((constants::STACK_SIZE) as u16)?;
        // There is no continuation, so push zero.
        self.push_addr(0)?;
        // No return variable, so just push Global 0xef.
        self.push_byte(u8::from(ZVariable::Global(0xef)))?;
        // There are no locals.
        self.push_byte(0)
    }

    pub fn saved_fp(&self) -> usize {
        usize::from(bytes::word_from_slice(
            &self.stack,
            self.fp + ZStack::SAVED_PC_OFFSET,
        ))
    }

    pub fn num_locals(&self) -> u8 {
        bytes::byte_from_slice(&self.stack, self.fp + ZStack::NUM_LOCALS_OFFSET).into()
    }

    fn push_addr(&mut self, addr: usize) -> Result<()> {
        // This should probably be a ZOffset.
        self.push_word((addr >> 16 & 0xffff) as u16)?;
        self.push_word((addr >> 0 & 0xffff) as u16)?;
        Ok(())
    }
}

impl Stack for ZStack {
    fn push_byte(&mut self, byte: u8) -> Result<()> {
        if self.sp < constants::STACK_SIZE {
            self.stack[self.sp] = byte;
            self.sp += 1;
            Ok(())
        } else {
            Err(ZErr::StackOverflow("Pushed bytes off end of stack."))
        }
    }

    fn pop_byte(&mut self) -> Result<u8> {
        if self.sp > self.s0 {
            self.sp -= 1;
            Ok(self.stack[self.sp])
        } else {
            Err(ZErr::StackUnderflow("Popped byte off empty stack."))
        }
    }

    fn read_local(&self, l: u8) -> Result<u16> {
        if l >= self.num_locals() {
            Err(ZErr::LocalOutOfRange(l, self.num_locals()))
        } else {
            Ok(bytes::word_from_slice(
                &self.stack,
                self.fp + ZStack::LOCAL_VAR_OFFSET + usize::from(l) * 2,
            ))
        }
    }

    fn write_local(&mut self, l: u8, val: u16) -> Result<()> {
        bytes::word_to_slice(
            &mut self.stack,
            self.fp + ZStack::LOCAL_VAR_OFFSET + usize::from(l) * 2,
            val,
        );
        Ok(())
    }

    fn return_pc(&self) -> usize {
        bytes::long_word_from_slice(&self.stack, self.fp + ZStack::RETURN_PC_OFFSET) as usize
    }

    fn return_variable(&self) -> ZVariable {
        bytes::byte_from_slice(&self.stack, self.fp + ZStack::RETURN_VAR_OFFSET).into()
    }

    fn push_frame(
        &mut self,
        return_pc: usize,
        num_locals: u8,
        return_var: ZVariable,
        operands: &[u16],
    ) -> Result<()> {
        // Steps:
        // - save sp to new_fp
        // - push fp
        // - save new_fp to fp
        // - push return_pc
        // - push return_var
        // - push num_locals
        // - push space for each local variable (initted to 0)
        // - set locals from operands
        // - set stack bottom to stack_next.
        let new_fp = self.sp;
        let old_fp = self.fp;
        self.push_word(old_fp as u16)?;
        self.fp = new_fp;
        self.push_addr(return_pc)?;
        // TODO: figure out that AsRef thing here.
        self.push_byte(u8::from(return_var))?;
        self.push_byte(num_locals)?;
        for _ in 0..num_locals {
            self.push_word(0)?;
        }

        for (idx, op) in operands.iter().enumerate() {
            if idx >= num_locals.into() {
                // TODO: probably want a warning here.
                break;
            }
            self.write_local(idx as u8, *op)?;
        }

        self.s0 = self.sp;
        Ok(())
    }

    fn pop_frame(&mut self) -> Result<()> {
        // Steps:
        // - Remember current fp (call it old_fp).
        // - Set fp to value from frame.
        // - Set sp to old_fp.
        // - Compute new value of s0.

        // Check for underflow.
        if self.saved_fp() >= constants::STACK_SIZE {
            return Err(ZErr::StackUnderflow("Popped top stack frame."));
        }

        let old_fp = self.fp;
        self.sp = old_fp;
        let saved_fp = self.saved_fp();
        self.fp = saved_fp;

        self.s0 = self.fp + ZStack::LOCAL_VAR_OFFSET + 2 * usize::from(self.num_locals());

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use zmachine::result::ZErr;

    #[test]
    fn test_new() {
        let stack = ZStack::new();
        let bytes = &stack.stack;

        assert_eq!(0, stack.fp);

        // Each frame is 8 bytes, so sp/s0 point to next byte.
        assert_eq!(8, stack.sp);
        assert_eq!(8, stack.s0);

        // FP should point to invalid value (one past the end of the stack array).
        let fp = usize::from(bytes[0]) << 8 + usize::from(bytes[1]);
        assert_eq!(constants::STACK_SIZE, fp);

        // return_pc should be 0
        let return_pc = usize::from(bytes[2])
            << 24 + usize::from(bytes[3])
            << 16 + usize::from(bytes[4])
            << 8 + usize::from(bytes[5]);
        assert_eq!(0, return_pc);

        // return value is Global 0xef.
        assert_eq!(u8::from(ZVariable::Global(0xef)), bytes[6]);

        // and there are no locals
        assert_eq!(0, bytes[7]);
    }

    #[test]
    fn test_push_frame() {
        let mut stack = ZStack::new();

        let old_fp = stack.fp;

        stack
            .push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38])
            .unwrap();

        assert_eq!(old_fp, stack.saved_fp());
        assert_eq!(0xbabef00d, stack.return_pc());
        assert_eq!(ZVariable::Global(3), stack.return_variable());
        assert_eq!(5, stack.num_locals());
        assert_eq!(34, stack.read_local(0).unwrap());
        assert_eq!(38, stack.read_local(1).unwrap());
        assert_eq!(0, stack.read_local(2).unwrap());
        assert_eq!(0, stack.read_local(3).unwrap());
        assert_eq!(0, stack.read_local(4).unwrap());
    }

    #[test]
    fn test_push_too_many_operands() {
        let mut stack = ZStack::new();

        stack
            .push_frame(0xbabef00d, 2, ZVariable::Stack, &[11, 24, 36, 48])
            .unwrap();

        assert_eq!(2, stack.num_locals());
        assert_eq!(11, stack.read_local(0).unwrap());
        assert_eq!(24, stack.read_local(1).unwrap());
    }

    #[test]
    fn test_local_range_check() {
        let mut stack = ZStack::new();

        stack
            .push_frame(0xbabef00d, 1, ZVariable::Stack, &[22])
            .unwrap();

        assert_eq!(22, stack.read_local(0).unwrap());
        match stack.read_local(1) {
            Err(ZErr::LocalOutOfRange(1, 1)) => (),
            Err(e) => panic!("Wrong error: {:?}", e),
            _ => panic!("No error when error expected."),
        }
    }

    #[test]
    fn test_push_and_pop_frame() {
        let mut stack = ZStack::new();

        let saved_fp1 = stack.fp;
        stack
            .push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38])
            .unwrap();

        let saved_fp2 = stack.fp;
        stack
            .push_frame(0x12345678, 7, ZVariable::Local(5), &[1, 3, 5])
            .unwrap();

        assert_eq!(saved_fp2, stack.saved_fp());
        assert_eq!(0x12345678, stack.return_pc());
        assert_eq!(ZVariable::Local(5), stack.return_variable());
        assert_eq!(7, stack.num_locals());
        assert_eq!(1, stack.read_local(0).unwrap());
        assert_eq!(3, stack.read_local(1).unwrap());
        assert_eq!(5, stack.read_local(2).unwrap());
        assert_eq!(0, stack.read_local(3).unwrap());
        assert_eq!(0, stack.read_local(4).unwrap());
        assert_eq!(0, stack.read_local(5).unwrap());
        assert_eq!(0, stack.read_local(6).unwrap());

        stack.pop_frame().unwrap();

        assert_eq!(saved_fp1, stack.saved_fp());
        assert_eq!(0xbabef00d, stack.return_pc());
        assert_eq!(ZVariable::Global(3), stack.return_variable());
        assert_eq!(5, stack.num_locals());
        assert_eq!(34, stack.read_local(0).unwrap());
        assert_eq!(38, stack.read_local(1).unwrap());
        assert_eq!(0, stack.read_local(2).unwrap());
        assert_eq!(0, stack.read_local(3).unwrap());
        assert_eq!(0, stack.read_local(4).unwrap());

        stack.pop_frame().unwrap();
    }

    #[test]
    fn test_push_pop_stack_values() {
        let mut stack = ZStack::new();

        stack
            .push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38])
            .unwrap();
        stack.push_word(34).unwrap();
        stack.push_word(4832).unwrap();
        stack.push_word(137).unwrap();

        stack
            .push_frame(0x12345678, 7, ZVariable::Local(5), &[1, 3, 5])
            .unwrap();
        stack.push_word(99).unwrap();
        stack.push_word(1293).unwrap();
        stack.push_word(44444).unwrap();
        stack.push_word(253).unwrap();

        assert_eq!(253, stack.pop_word().unwrap());
        assert_eq!(44444, stack.pop_word().unwrap());
        assert_eq!(1293, stack.pop_word().unwrap());
        assert_eq!(99, stack.pop_word().unwrap());

        // TODO: test for underflow

        stack.pop_frame().unwrap();

        assert_eq!(137, stack.pop_word().unwrap());
        assert_eq!(4832, stack.pop_word().unwrap());
        assert_eq!(34, stack.pop_word().unwrap());
    }

    #[test]
    fn test_pop_missing_stack_frame() {
        let mut stack = ZStack::new();

        match stack.pop_frame() {
            Err(ZErr::StackUnderflow(_)) => {}
            _ => panic!("Missing error"),
        }
    }

    #[test]
    fn test_stack_frame_overflow() {
        let mut stack = ZStack::new();

        // 42 stack frames is as many as fit on the current sized frame.
        for _ in 0..42 {
            stack.push_frame(0x1000, 8, ZVariable::Stack, &[]).unwrap();
        }

        match stack.push_frame(0x2000, 8, ZVariable::Stack, &[]) {
            Err(ZErr::StackOverflow(_)) => {}
            Err(e) => panic!("Wrong error: {:?}", e),
            Ok(_) => panic!("Missing error"),
        }
    }

    #[test]
    fn test_stack_overflow() {
        let mut stack = ZStack::new();

        // 42 stack frames is as many as fit on the current sized frame.
        for _ in 0..42 {
            stack.push_frame(0x1000, 8, ZVariable::Stack, &[]).unwrap();
        }

        // Then, we can fit 4 more words.
        stack.push_word(3).unwrap();
        stack.push_word(4).unwrap();
        stack.push_word(5).unwrap();
        stack.push_word(6).unwrap();

        match stack.push_word(7) {
            Err(ZErr::StackOverflow(_)) => {}
            Err(e) => panic!("Wrong error: {:?}", e),
            Ok(_) => panic!("Missing error"),
        }
    }

    #[test]
    fn test_stack_underflow_after_popping_frame() {
        let mut stack = ZStack::new();

        stack.push_word(4).unwrap();
        stack.push_word(4).unwrap();

        let old_s0 = stack.s0;
        assert_eq!(stack.sp, stack.s0 + 4);

        stack
            .push_frame(0xabcdef00, 4, ZVariable::Stack, &[])
            .unwrap();
        stack.pop_frame().unwrap();

        assert_eq!(old_s0, stack.s0);

        stack.pop_word().unwrap();
        stack.pop_word().unwrap();

        match stack.pop_byte() {
            Err(ZErr::StackUnderflow(_)) => {}
            Err(e) => panic!("Wrong error: {:?}", e),
            Ok(_) => panic!("Missing error"),
        }
    }
}
