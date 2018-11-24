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

pub struct PC(usize, ZVersion);

impl PC {
    pub fn new(pa: PackedAddress, vers: ZVersion) -> PC {
        match vers {
            ZVersion::V3 => PC(2 * pa.0 as usize, vers),
            ZVersion::V5 => PC(4 * pa.0 as usize, vers),
        }
    }

    pub fn inc(&mut self) {
        self.inc_by(1);
    }

    pub fn inc_by(&mut self, increment: i16) {
        if increment < 0 {
            self.0 -= -increment as usize;
        } else {
            self.0 += increment as usize;
        }
    }
}

impl From<PC> for ZOffset {
    fn from(pc: PC) -> ZOffset {
        ZOffset(pc.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pc_inc() {
        let mut pc = PC(8, ZVersion::V3);
        assert_eq!(8, pc.0);
        pc.inc();
        assert_eq!(9, pc.0);
        pc.inc();
        assert_eq!(10, pc.0);
    }

    #[test]
    fn test_pc_inc_by() {
        let mut pc = PC(13, ZVersion::V3);
        assert_eq!(13, pc.0);

        pc.inc_by(5);
        assert_eq!(18, pc.0);

        pc.inc_by(-3);
        assert_eq!(15, pc.0);
    }

    #[test]
    fn test_pc_new_vers() {
        let pc3 = PC::new(PackedAddress(0xccdd), ZVersion::V3);
        assert_eq!(0x199ba, pc3.0);

        let pc5 = PC::new(PackedAddress(0xccdd), ZVersion::V5);
        assert_eq!(0x33374, pc5.0);
    }
}
