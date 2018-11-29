use std::fmt;

use super::handle::Handle;
use super::memory::ZMemory;
use super::traits::PC;
use super::version::ZVersion;

#[derive(Clone, Copy, Debug)]
pub struct ZOffset(usize);

impl ZOffset {
    pub fn inc_by(self, by: usize) -> ZOffset {
        ZOffset(self.0 + by)
    }

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

    pub fn inc_by(self, by: u16) -> ByteAddress {
        ByteAddress(self.0 + by)
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
pub struct PackedAddress {
    val: u16,
    multiplier: u8,
    offset: u16, // for V6 only, other versions set this to zero.
}

impl PackedAddress {
    pub fn new(val: u16, version: ZVersion) -> PackedAddress {
        let multiplier = match version {
            ZVersion::V3 => 2,
            ZVersion::V5 => 4,
        };
        PackedAddress {
            val,
            multiplier,
            offset: 0,
        }
    }
}

impl From<PackedAddress> for usize {
    fn from(pa: PackedAddress) -> usize {
        let offset = ZOffset::from(pa);
        offset.value()
    }
}

impl From<PackedAddress> for ZOffset {
    fn from(pa: PackedAddress) -> ZOffset {
        ZOffset(usize::from(pa.val) * usize::from(pa.multiplier) + usize::from(pa.offset))
    }
}

impl fmt::Display for PackedAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p{:x}", usize::from(*self))
    }
}

pub struct ZPC {
    pc: usize,
    mem_h: Handle<ZMemory>,
}

impl ZPC {
    pub fn new<T>(mem_h: &Handle<ZMemory>, start_pc: T) -> ZPC
    where
        T: Into<ZOffset>,
    {
        ZPC {
            pc: start_pc.into().0,
            mem_h: mem_h.clone(),
        }
    }
}

impl PC for ZPC {
    fn current_pc(&self) -> usize {
        self.pc
    }

    fn set_current_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
    }

    fn next_byte(&mut self) -> u8 {
        let offset = ZOffset(self.pc);
        let byte = self.mem_h.borrow().read_byte(offset);
        self.pc += 1;
        byte
    }
}

impl From<ZPC> for ZOffset {
    fn from(pc: ZPC) -> ZOffset {
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
        let pc3 = ZPC::new(
            &test_mem(ZVersion::V3),
            PackedAddress::new(0xccdd, ZVersion::V3),
        );
        // TODO: do this.
        //        assert_eq!(0x199ba, pc3.pc);

        // TODO: put this back in.
        //        let pc5 = PC::new(&test_mem(ZVersion::V3), PackedAddress(0xccdd), ZVersion::V5);
        //        assert_eq!(0x33374, pc5.pc);
    }
}
