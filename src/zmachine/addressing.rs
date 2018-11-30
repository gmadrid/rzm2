use std::fmt;

use super::handle::Handle;
use super::traits::{Memory, PC};
use super::version::ZVersion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl WordAddress {
    pub fn from_raw(word: u16) -> WordAddress {
        WordAddress(word)
    }
}

impl From<WordAddress> for ZOffset {
    fn from(ba: WordAddress) -> ZOffset {
        ZOffset(usize::from(ba.0) * 2)
    }
}

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

pub struct ZPC<M>
where
    M: Memory,
{
    pc: usize,
    mem_h: Handle<M>,
}

impl<M> ZPC<M>
where
    M: Memory,
{
    pub fn new<T>(mem_h: &Handle<M>, start_pc: T) -> ZPC<M>
    where
        T: Into<ZOffset>,
    {
        ZPC {
            pc: start_pc.into().0,
            mem_h: mem_h.clone(),
        }
    }
}

impl<M> PC for ZPC<M>
where
    M: Memory,
{
    fn current_pc(&self) -> usize {
        self.pc
    }

    fn set_current_pc(&mut self, new_pc: usize) {
        // TODO: check range.
        self.pc = new_pc;
    }

    fn next_byte(&mut self) -> u8 {
        // TODO: check range.
        let offset = ZOffset(self.pc);
        let byte = self.mem_h.borrow().read_byte(offset);
        self.pc += 1;
        byte
    }
}

impl<M> From<ZPC<M>> for ZOffset
where
    M: Memory,
{
    fn from(pc: ZPC<M>) -> ZOffset {
        ZOffset(pc.pc)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use zmachine::fixtures::TestMemory;
    use zmachine::handle::new_handle;

    #[test]
    fn test_zoffset() {
        let zo = ZOffset(48);
        assert_eq!(48, zo.value());

        assert_eq!(52, zo.inc_by(4).value())
    }

    #[test]
    fn test_byte_address() {
        let ba = ByteAddress::from_raw(58);
        assert_eq!(58, ZOffset::from(ba).value());
        assert_eq!(65, ZOffset::from(ba.inc_by(7)).value());
    }

    #[test]
    fn test_word_address() {
        let wa = WordAddress::from_raw(62);
        assert_eq!(124, ZOffset::from(wa).value());
    }

    #[test]
    fn test_packed_address() {
        let pa3 = PackedAddress::new(53, ZVersion::V3);
        assert_eq!(106, usize::from(pa3));
        assert_eq!(106, ZOffset::from(pa3).value());

        let pa5 = PackedAddress::new(53, ZVersion::V5);
        assert_eq!(212, usize::from(pa5));
        assert_eq!(212, ZOffset::from(pa5).value());
    }

    #[test]
    fn test_pc() {
        let test_mem = new_handle(TestMemory::new_from_vec(vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ]));
        let mut pc = ZPC::new(&test_mem, ZOffset(5));

        assert_eq!(5, pc.current_pc());
        pc.set_current_pc(9);
        assert_eq!(9, pc.current_pc());

        assert_eq!(9, pc.next_byte());
        assert_eq!(10, pc.next_byte());
        assert_eq!(11, pc.current_pc());

        assert_eq!(11, ZOffset::from(pc).value());
    }
}
