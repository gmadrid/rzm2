use super::handle::Handle;
use super::memory::ZMemory;
use super::version::ZVersion;

// A ZOffset is an index into ZMemory.
//
// It can reference the entire core memory. It cannot be created directly,
// but only from one of the addressing types: ByteAddress, WordAddress, or
// PackedAddress.
//
// The PC is represented as a raw ZOffset, since none of the addressing modes
// can access every byte in core memory.
#[derive(Clone, Copy, Debug)]
pub struct ZOffset(usize);

impl ZOffset {
    pub fn value(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ByteAddress(u16);

impl ByteAddress {
    pub fn from_raw(word: u16) -> ByteAddress {
        ByteAddress(word)
    }
}

impl From<ByteAddress> for ZOffset {
    fn from(ba: ByteAddress) -> ZOffset {
        ZOffset(ba.0 as usize)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WordAddress(u16);

#[derive(Clone, Copy)]
pub struct PackedAddress(u16);

impl PackedAddress {
    pub fn from_raw(word: u16) -> PackedAddress {
        PackedAddress(word)
    }
}

impl From<PackedAddress> for ZOffset {
    fn from(pa: PackedAddress) -> ZOffset {
        // TODO: this only works in V3! XXX
        ZOffset(pa.0 as usize * 2)
    }
}

pub struct PC {
    pc: usize,
    version: ZVersion,
    mem_h: Handle<ZMemory>,
}

impl PC {
    pub fn new<T>(mem_h: &Handle<ZMemory>, start_pc: T, version: ZVersion) -> PC
    where
        T: Into<ZOffset>,
    {
        PC {
            pc: start_pc.into().0,
            version,
            mem_h: mem_h.clone(),
        }
    }

    // pub fn inc(&mut self) {
    //     self.inc_by(1);
    // }

    // pub fn inc_by(&mut self, increment: i16) {
    //     if increment < 0 {
    //         self.pc -= -increment as usize;
    //     } else {
    //         self.pc += increment as usize;
    //     }
    // }

    pub fn next_byte(&mut self) -> u8 {
        let offset = ZOffset(self.pc);
        let byte = self.mem_h.read_byte(offset);
        self.pc += 1;
        byte
    }

    pub fn next_word(&mut self) -> u16 {
        let high_byte = self.next_byte();
        let low_byte = self.next_byte();
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

impl From<PC> for ZOffset {
    fn from(pc: PC) -> ZOffset {
        ZOffset(pc.pc)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    // #[test]
    // fn test_pc_inc() {
    //     let mut pc = PC(8, ZVersion::V3);
    //     assert_eq!(8, pc.0);
    //     pc.inc();
    //     assert_eq!(9, pc.0);
    //     pc.inc();
    //     assert_eq!(10, pc.0);
    // }

    // #[test]
    // fn test_pc_inc_by() {
    //     let mut pc = PC(13, ZVersion::V3);
    //     assert_eq!(13, pc.0);

    //     pc.inc_by(5);
    //     assert_eq!(18, pc.0);

    //     pc.inc_by(-3);
    //     assert_eq!(15, pc.0);
    // }

    fn sample_bytes() -> Vec<u8> {
        vec![3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 0xcc, 0xdd]
    }

    fn test_mem(vers: ZVersion) -> Handle<ZMemory> {
        let mut bytes = sample_bytes();
        bytes[0] = vers as u8;
        ZMemory::new(&mut Cursor::new(&bytes)).unwrap().0
    }

    #[test]
    fn test_pc_new_vers() {
        let pc3 = PC::new(&test_mem(ZVersion::V3), PackedAddress(0xccdd), ZVersion::V3);
        assert_eq!(0x199ba, pc3.pc);

        // TODO: put this back in.
//        let pc5 = PC::new(&test_mem(ZVersion::V3), PackedAddress(0xccdd), ZVersion::V5);
//        assert_eq!(0x33374, pc5.pc);
    }
}
