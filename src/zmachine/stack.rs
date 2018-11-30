// UNREVIEWED

use super::constants;
use super::opcode::ZVariable;
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

        //
        // Create a pseudo-frame for the base frame.
        //

        // There is not previous frame, so point to an illegal value.
        zs.push_word((constants::STACK_SIZE) as u16);
        // There is no continuation, so push zero.
        zs.push_addr(0);
        // No return variable, so just push Global 0xef.
        zs.push_byte(u8::from(ZVariable::Global(0xef)));
        // There are no locals.
        zs.push_byte(0);

        zs.s0 = zs.sp;

        zs
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

    fn push_addr(&mut self, addr: usize) {
        // This should probably be a ZOffset.
        self.push_word((addr >> 16 & 0xffff) as u16);
        self.push_word((addr >> 0 & 0xffff) as u16);
    }
}

impl Stack for ZStack {
    fn push_byte(&mut self, byte: u8) {
        self.stack[self.sp] = byte;
        self.sp += 1;
    }

    fn pop_byte(&mut self) -> u8 {
        self.sp -= 1;
        self.stack[self.sp]
    }

    fn read_local(&self, l: u8) -> u16 {
        bytes::word_from_slice(
            &self.stack,
            self.fp + ZStack::LOCAL_VAR_OFFSET + usize::from(l) * 2,
        )
    }

    fn write_local(&mut self, l: u8, val: u16) {
        bytes::word_to_slice(
            &mut self.stack,
            self.fp + ZStack::LOCAL_VAR_OFFSET + usize::from(l) * 2,
            val,
        );
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
    ) {
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
        self.push_word(old_fp as u16);
        self.fp = new_fp;
        self.push_addr(return_pc);
        // TODO: figure out that AsRef thing here.
        self.push_byte(u8::from(return_var));
        self.push_byte(num_locals);
        for _ in 0..num_locals {
            self.push_word(0);
        }

        for (idx, op) in operands.iter().enumerate() {
            if idx >= num_locals.into() {
                // TODO: probably want a warning here.
                break;
            }
            self.write_local(idx as u8, *op);
        }

        self.s0 = self.sp;
    }

    fn pop_frame(&mut self) {
        // Steps:
        // - Remember current fp (call it old_fp).
        // - Set fp to value from frame.
        // - Set sp to old_fp.
        // - Compute new value of s0.
        let old_fp = self.fp;
        self.sp = old_fp;
        let saved_fp = self.saved_fp();
        println!("saved fp: {}", saved_fp);
        self.fp = saved_fp;
        // TODO: make sure you haven't underflowed.

        // What is s0 right now?
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

        stack.push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38]);

        assert_eq!(0xbabef00d, stack.return_pc());
        assert_eq!(ZVariable::Global(3), stack.return_variable());
        assert_eq!(5, stack.num_locals());
        assert_eq!(34, stack.read_local(0));
        assert_eq!(38, stack.read_local(1));
        assert_eq!(0, stack.read_local(2));
        assert_eq!(0, stack.read_local(3));
        assert_eq!(0, stack.read_local(4));
        // TODO: add a test here for reading off the end of the variables list.
    }

    #[test]
    fn test_push_and_pop_frame() {
        let mut stack = ZStack::new();

        stack.push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38]);
        stack.push_frame(0x12345678, 7, ZVariable::Local(5), &[1, 3, 5]);

        assert_eq!(0x12345678, stack.return_pc());
        assert_eq!(ZVariable::Local(5), stack.return_variable());
        assert_eq!(7, stack.num_locals());
        assert_eq!(1, stack.read_local(0));
        assert_eq!(3, stack.read_local(1));
        assert_eq!(5, stack.read_local(2));
        assert_eq!(0, stack.read_local(3));
        assert_eq!(0, stack.read_local(4));
        assert_eq!(0, stack.read_local(5));
        assert_eq!(0, stack.read_local(6));

        stack.pop_frame();

        assert_eq!(0xbabef00d, stack.return_pc());
        assert_eq!(ZVariable::Global(3), stack.return_variable());
        assert_eq!(5, stack.num_locals());
        assert_eq!(34, stack.read_local(0));
        assert_eq!(38, stack.read_local(1));
        assert_eq!(0, stack.read_local(2));
        assert_eq!(0, stack.read_local(3));
        assert_eq!(0, stack.read_local(4));
    }

    #[test]
    fn test_push_pop_stack_values() {
        let mut stack = ZStack::new();

        stack.push_frame(0xbabef00d, 5, ZVariable::Global(3), &[34, 38]);
        stack.push_word(34);
        stack.push_word(4832);
        stack.push_word(137);

        stack.push_frame(0x12345678, 7, ZVariable::Local(5), &[1, 3, 5]);
        stack.push_word(99);
        stack.push_word(1293);
        stack.push_word(44444);
        stack.push_word(253);

        assert_eq!(253, stack.pop_word());
        assert_eq!(44444, stack.pop_word());
        assert_eq!(1293, stack.pop_word());
        assert_eq!(99, stack.pop_word());

        // TODO: test for underflow

        stack.pop_frame();

        assert_eq!(137, stack.pop_word());
        assert_eq!(4832, stack.pop_word());
        assert_eq!(34, stack.pop_word());
    }

    // TODO: add a test for having more operands than locals.

}
