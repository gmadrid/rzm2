use super::addressing::ZOffset;
use super::opcode::ZVariable;
use super::version::ZVersion;

pub mod bytes {
    #[inline]
    pub fn byte_from_slice(slice: &[u8], idx: usize) -> u8 {
        slice[idx]
    }

    #[inline]
    pub fn byte_to_slice(slice: &mut [u8], idx: usize, val: u8) {
        slice[idx] = val;
    }

    pub fn word_from_slice(slice: &[u8], idx: usize) -> u16 {
        // big-endian
        let high_byte = u16::from(byte_from_slice(slice, idx));
        let low_byte = u16::from(byte_from_slice(slice, idx + 1));

        (high_byte << 8) + low_byte
    }

    pub fn word_to_slice(slice: &mut [u8], idx: usize, val: u16) {
        let high_byte = ((val >> 8) & 0xff) as u8;
        let low_byte = (val & 0xff) as u8;

        // big-endian
        byte_to_slice(slice, idx, high_byte);
        byte_to_slice(slice, idx + 1, low_byte);
    }
}

pub trait Header {
    fn version_number(&self) -> ZVersion;
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn next_byte(&mut self) -> u8;

    fn next_word(&mut self) -> u16 {
        let high_byte = self.next_byte();
        let low_byte = self.next_byte();
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

pub trait Memory {
    fn get_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy;

    fn set_byte<T>(&mut self, at: T, val: u8)
    where
        T: Into<ZOffset> + Copy;

    fn get_word<T>(&self, at: T) -> u16
    where
        T: Into<ZOffset> + Copy,
    {
        let high_byte = u16::from(self.get_byte(at.into()));
        let low_byte = u16::from(self.get_byte(at.into().inc_by(1)));
        (high_byte << 8) + low_byte
    }

    fn set_word<T>(&mut self, at: T, val: u16)
    where
        T: Into<ZOffset> + Copy,
    {
        let high_byte = ((val >> 8) & 0xff) as u8;
        let low_byte = (val & 0xff) as u8;
        let offset = at.into();
        self.set_byte(offset, high_byte);
        self.set_byte(offset.inc_by(1), low_byte);
    }
}

pub trait Stack {
    fn push_byte(&mut self, val: u8);
    fn pop_byte(&mut self) -> u8;

    fn read_local(&self, l: u8) -> u16;
    fn write_local(&self, l: u8, val: u16);

    fn push_word(&mut self, word: u16) {
        self.push_byte((word >> 8 & 0xff) as u8);
        self.push_byte((word >> 0 & 0xff) as u8);
    }

    fn pop_word(&mut self) -> u16 {
        let low_byte = u16::from(self.pop_byte());
        let high_byte = u16::from(self.pop_byte());

        (high_byte << 8) + low_byte
    }
}

pub trait Variables {
    fn read_variable(&self, var: ZVariable) -> u16;
    fn write_variable(&mut self, var: ZVariable, val: u16);
}

#[cfg(test)]
mod test {
    use super::super::addressing::ByteAddress;
    use super::*;

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
        val: u8,
    }

    impl PC for TestPC {
        fn current_pc(&self) -> usize {
            0x89
        }

        fn next_byte(&mut self) -> u8 {
            self.val += 1;
            self.val
        }
    }

    #[test]
    fn test_pc_default_implementations() {
        let mut pc = TestPC { val: 0x78 };

        assert_eq!(0x797a, pc.next_word());
        assert_eq!(0x7b7c, pc.next_word());
    }

    struct TestMemory {
        val: [u8; 16],
    }

    impl Memory for TestMemory {
        fn get_byte<T>(&self, at: T) -> u8
        where
            T: Into<ZOffset> + Copy,
        {
            self.val[at.into().value()]
        }

        fn set_byte<T>(&mut self, at: T, val: u8)
        where
            T: Into<ZOffset> + Copy,
        {
            self.val[at.into().value()] = val;
        }
    }

    #[test]
    fn test_memory_default_implementations() {
        let mut arr = [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        let mut memory = TestMemory { val: arr };

        assert_eq!(0x0405, memory.get_word(ByteAddress::from_raw(1)));
        assert_eq!(0x1011, memory.get_word(ByteAddress::from_raw(13)));

        memory.set_word(ByteAddress::from_raw(1), 0x89ab);

        // now: [3, 0x89, 0xab, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        assert_eq!(0x0389, memory.get_word(ByteAddress::from_raw(0)));
        assert_eq!(0x89ab, memory.get_word(ByteAddress::from_raw(1)));
        assert_eq!(0xab06, memory.get_word(ByteAddress::from_raw(2)));
    }

    #[derive(Default)]
    struct TestStack {
        arr: Vec<u8>, // a very small stack.
    }

    impl Stack for TestStack {
        fn push_byte(&mut self, val: u8) {
            self.arr.push(val);
        }
        fn pop_byte(&mut self) -> u8 {
            self.arr.pop().unwrap()
        }

        fn read_local(&self, l: u8) -> u16 {
            0
        }
        fn write_local(&self, l: u8, val: u16) {}
    }

    #[test]
    fn test_stack_default_implementations() {
        let mut stack = TestStack::default();

        stack.push_byte(0x01);
        stack.push_word(0x0203);
        stack.push_byte(0x04);

        assert_eq!(0x0304, stack.pop_word());
        assert_eq!(0x02, stack.pop_byte());
        assert_eq!(0x01, stack.pop_byte());
    }
}
