use super::constants;
use super::opcode::ZVariable;
use super::traits::Stack;

// Stack size maxes out at 1024.
//
// Stack frame:
//   frame ptr (u16)    - index in the stack of the previous frame
//   return PC (usize)  - pc of continuation after this call returns
//   num_locals (u8)    - number of words for local variables (and call params)
//   N * local
//   eval stack

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

    pub fn push_frame(
        &mut self,
        return_pc: usize,
        num_locals: u8,
        return_var: &ZVariable,
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
        self.push_byte(u8::from(*return_var));
        self.push_byte(num_locals);
        for _ in 0..num_locals {
            self.push_word(0);
        }

        for (idx, op) in operands.iter().enumerate() {
            if idx >= num_locals.into() {
                // TODO: probably want a warning here.
                break;
            }
            self.set_local(idx, *op);
        }

        self.s0 = self.sp;
    }

    fn push_addr(&mut self, addr: usize) {
        // This should probably be a ZOffset.
        self.push_word((addr >> 16 & 0xffff) as u16);
        self.push_word((addr >> 0 & 0xffff) as u16);
    }

    fn set_local(&mut self, idx: usize, val: u16) {
        panic!("UNIMPLEMENTED");
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
        panic!("unimplemented")
    }

    fn write_local(&self, l: u8, val: u16) {
        panic!("unimplemented")
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
}
