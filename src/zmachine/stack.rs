use super::constants;
use super::traits::Stack;

// Stack size maxes out at 1024.
//
// Stack frame:
//   frame ptr (u16)     - offset in the stack of the previous frame
//   return PC (usize)  - pc of continuation after this call returns
//   num_locals (u8)    - number of words for local variables (and call params)
//   N * local
//   eval stack

pub struct ZStack {
    stack: [u8; constants::STACK_SIZE],
    fp: usize,
    sp: usize, // points to the next empty byte
}

impl ZStack {
    pub fn new() -> ZStack {
        let mut zs = ZStack {
            stack: [0; constants::STACK_SIZE],
            fp: 0,
            sp: 0,
        };

        //
        // Create a pseudo-frame for the base frame.
        //

        // There is not previous frame, so point to an illegal value.
        zs.push_word((constants::STACK_SIZE + 1) as u16);
        // There is no continuation, so push zero.
        zs.push_addr(0);
        // There are no locals.
        zs.push_byte(0);

        zs
    }

    fn push_byte(&mut self, byte: u8) {
        self.stack[self.sp] = byte;
        self.sp += 1;
    }

    fn push_word(&mut self, word: u16) {
        self.push_byte((word >> 8 & 0xff) as u8);
        self.push_byte((word >> 0 & 0xff) as u8);
    }

    fn push_addr(&mut self, addr: usize) {
        // This should probably be a ZOffset.
        self.push_word((addr >> 16 & 0xffff) as u16);
        self.push_word((addr >> 0 & 0xffff) as u16);
    }
}

impl Stack for ZStack {}
