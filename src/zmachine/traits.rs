use super::addressing::{ByteAddress, ZOffset};
use super::opcode::ZVariable;
use super::result::Result;
use super::version::ZVersion;

pub mod bytes {
    // TODO: range check all of this.

    #[inline]
    pub fn byte_from_slice(slice: &[u8], idx: usize) -> u8 {
        slice[idx]
    }

    #[inline]
    pub fn byte_to_slice(slice: &mut [u8], idx: usize, val: u8) {
        slice[idx] = val;
    }

    #[inline]
    pub fn word_from_slice(slice: &[u8], idx: usize) -> u16 {
        // big-endian
        let high_byte = u16::from(byte_from_slice(slice, idx));
        let low_byte = u16::from(byte_from_slice(slice, idx + 1));

        (high_byte << 8) + low_byte
    }

    #[inline]
    pub fn word_to_slice(slice: &mut [u8], idx: usize, val: u16) {
        let high_byte = ((val >> 8) & 0xff) as u8;
        let low_byte = (val & 0xff) as u8;

        // big-endian
        byte_to_slice(slice, idx, high_byte);
        byte_to_slice(slice, idx + 1, low_byte);
    }

    #[inline]
    pub fn long_word_from_slice(slice: &[u8], idx: usize) -> u32 {
        // big-endian
        let byte_3 = u32::from(byte_from_slice(slice, idx));
        let byte_2 = u32::from(byte_from_slice(slice, idx + 1));
        let byte_1 = u32::from(byte_from_slice(slice, idx + 2));
        let byte_0 = u32::from(byte_from_slice(slice, idx + 3));

        (byte_3 << 24) + (byte_2 << 16) + (byte_1 << 8) + byte_0
    }
}

pub trait Header {
    fn global_location(&self) -> ByteAddress;
    fn high_memory_base(&self) -> ByteAddress;
    fn static_memory_base(&self) -> ByteAddress;
    fn version_number(&self) -> ZVersion;
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn set_current_pc(&mut self, new_pc: usize);
    fn next_byte(&mut self) -> u8;

    fn offset_pc(&mut self, offset: isize) {
        // TODO: check for underflow.
        let pc = self.current_pc() as isize;
        self.set_current_pc((pc + offset) as usize);
    }

    fn next_word(&mut self) -> u16 {
        let high_byte = self.next_byte();
        let low_byte = self.next_byte();
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

pub trait Memory {
    fn read_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy;

    fn write_byte<T>(&mut self, at: T, val: u8) -> Result<()>
    where
        T: Into<ZOffset> + Copy;

    fn read_word<T>(&self, at: T) -> u16
    where
        T: Into<ZOffset> + Copy,
    {
        let high_byte = u16::from(self.read_byte(at.into()));
        let low_byte = u16::from(self.read_byte(at.into().inc_by(1)));
        (high_byte << 8) + low_byte
    }

    // May fail if word is outside dynamic memory.
    fn write_word<T>(&mut self, at: T, val: u16) -> Result<()>
    where
        T: Into<ZOffset> + Copy,
    {
        let high_byte = ((val >> 8) & 0xff) as u8;
        let low_byte = (val & 0xff) as u8;
        let offset = at.into();
        self.write_byte(offset, high_byte)?;
        self.write_byte(offset.inc_by(1), low_byte)
    }
}

pub trait Stack {
    fn push_byte(&mut self, val: u8) -> Result<()>;
    fn pop_byte(&mut self) -> Result<u8>;

    fn read_local(&self, l: u8) -> Result<u16>;
    fn write_local(&mut self, l: u8, val: u16) -> Result<()>;

    fn push_frame(
        &mut self,
        return_pc: usize,
        num_locals: u8,
        return_var: ZVariable,
        operands: &[u16],
    ) -> Result<()>;
    fn pop_frame(&mut self) -> Result<()>;

    fn return_pc(&self) -> usize;
    fn return_variable(&self) -> ZVariable;

    fn push_word(&mut self, word: u16) -> Result<()> {
        self.push_byte((word >> 8 & 0xff) as u8)?;
        self.push_byte((word >> 0 & 0xff) as u8)?;
        Ok(())
    }

    fn pop_word(&mut self) -> Result<u16> {
        let low_byte = u16::from(self.pop_byte()?);
        let high_byte = u16::from(self.pop_byte()?);

        Ok((high_byte << 8) + low_byte)
    }
}

pub trait Variables {
    // NOTE: read_variable requires a 'mut' self because reading from the Stack
    // causes a mutation.
    fn read_variable(&mut self, var: ZVariable) -> Result<u16>;

    // TODO: range check variable sub-values. (MAX_LOCAL, MAX_GLOBAL)
    fn write_variable(&mut self, var: ZVariable, val: u16) -> Result<()>;
}

#[cfg(test)]
mod test {
    use super::*;
    use zmachine::addressing::ByteAddress;
    use zmachine::result::ZErr;

    #[test]
    fn test_bytes() {
        let mut arr = [3, 4, 5, 6, 7, 8, 9, 10];

        assert_eq!(5, bytes::byte_from_slice(&arr, 2));
        assert_eq!(8, bytes::byte_from_slice(&arr, 5));

        bytes::byte_to_slice(&mut arr, 2, 0x89);

        // now: [3, 4, 0x89, 6, 7, 8, 9, 10];
        assert_eq!(0x89, bytes::byte_from_slice(&arr, 2));
        assert_eq!(8, bytes::byte_from_slice(&arr, 5));

        assert_eq!(0x0489, bytes::word_from_slice(&arr, 1));

        bytes::word_to_slice(&mut arr, 2, 0x5678);

        // now: [3, 4, 0x56, 0x78, 7, 8, 9, 10];
        assert_eq!(0x0456, bytes::word_from_slice(&arr, 1));
        assert_eq!(0x5678, bytes::word_from_slice(&arr, 2));
        assert_eq!(0x7807, bytes::word_from_slice(&arr, 3));
    }

    struct TestPC {
        val: usize,
    }

    impl PC for TestPC {
        fn current_pc(&self) -> usize {
            self.val
        }

        fn next_byte(&mut self) -> u8 {
            self.val += 1;
            self.val as u8
        }

        fn set_current_pc(&mut self, new_pc: usize) {
            self.val = new_pc;
        }
    }

    #[test]
    fn test_pc_default_implementations() {
        let mut pc = TestPC { val: 0x78 };

        assert_eq!(0x797a, pc.next_word());
        assert_eq!(0x7b7c, pc.next_word());

        let mut pc = TestPC { val: 0x78 };
        pc.offset_pc(16);
        assert_eq!(0x88, pc.current_pc());

        pc.offset_pc(-8);
        assert_eq!(0x80, pc.current_pc());
    }

    struct TestMemory {
        val: [u8; 16],
    }

    impl Memory for TestMemory {
        fn read_byte<T>(&self, at: T) -> u8
        where
            T: Into<ZOffset> + Copy,
        {
            self.val[at.into().value()]
        }

        fn write_byte<T>(&mut self, at: T, val: u8) -> Result<()>
        where
            T: Into<ZOffset> + Copy,
        {
            self.val[at.into().value()] = val;
            Ok(())
        }
    }

    #[test]
    fn test_memory_default_implementations() {
        let arr = [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        let mut memory = TestMemory { val: arr };

        assert_eq!(0x0405, memory.read_word(ByteAddress::from_raw(1)));
        assert_eq!(0x1011, memory.read_word(ByteAddress::from_raw(13)));

        memory.write_word(ByteAddress::from_raw(1), 0x89ab).unwrap();

        // now: [3, 0x89, 0xab, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        assert_eq!(0x0389, memory.read_word(ByteAddress::from_raw(0)));
        assert_eq!(0x89ab, memory.read_word(ByteAddress::from_raw(1)));
        assert_eq!(0xab06, memory.read_word(ByteAddress::from_raw(2)));
    }

    // A Stack implementation that doesn't re-implement any of the default fns.
    #[derive(Default)]
    struct BareStack {
        arr: Vec<u8>, // a very small stack.
    }

    impl Stack for BareStack {
        fn push_byte(&mut self, val: u8) -> Result<()> {
            self.arr.push(val);
            Ok(())
        }
        fn pop_byte(&mut self) -> Result<u8> {
            self.arr.pop().ok_or(ZErr::GenericError("Popping in BareStack"))
        }

        fn pop_frame(&mut self) -> Result<()> {
            Ok(())
        }
        fn return_pc(&self) -> usize {
            panic!("unimplemented")
        }
        fn return_variable(&self) -> ZVariable {
            panic!("unimplemented")
        }

        fn read_local(&self, _l: u8) -> Result<u16> {
            Ok(0)
        }
        fn write_local(&mut self, _l: u8, _val: u16) -> Result<()> {
            Ok(())
        }

        fn push_frame(
            &mut self,
            _return_pc: usize,
            _num_locals: u8,
            _return_var: ZVariable,
            _operands: &[u16],
        ) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_stack_default_implementations() {
        let mut stack = BareStack::default();

        stack.push_byte(0x01).unwrap();
        stack.push_word(0x0203).unwrap();
        stack.push_byte(0x04).unwrap();

        assert_eq!(0x0304, stack.pop_word().unwrap());
        assert_eq!(0x02, stack.pop_byte().unwrap());
        assert_eq!(0x01, stack.pop_byte().unwrap());
    }
}
