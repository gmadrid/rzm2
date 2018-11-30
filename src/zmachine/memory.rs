use std::io::Read;

use super::addressing::{ByteAddress, ZOffset};
use super::handle::{new_handle, Handle};
use super::header::{self, ZHeader};
use super::result::{Result, ZErr};
use super::traits::{bytes, Header, Memory};

// The "core memory" of the ZMachine. A memory-mapped story file.
//
// ZMemory provides protected access to the ZMachine's core memory.
// Only the dynamic portion of the ZMemory is available for read/write access.
// The static and high portions of memory are available for reading.
//
// Addresses may be specified using any of three types of addresses:
//
//   ByteAddress: access to any of the first 64K bytes in the core memory.
//
//   WordAddress: access to any of the first 64K words (so, 128K bytes) in core
//     memory.
//
//   PackedAddress: used to reference high memory. The extent and interpretation
//     of a PackedAddress changes depending on the ZMachine version in use.
//
pub struct ZMemory {
    bytes: Box<[u8]>,

    static_mem: ZOffset, // Offset of the base of static memory.
    high_mem: ZOffset,   // Offset of the base of high memory.
}

impl ZMemory {
    // Read initial memory state from a reader. The size of the ZMemory will be
    // determined by the length of the data read from the reader.
    //
    // The entire reader will be consumed to create the ZMemory.
    //
    // An error may be returned if the reader produces an error.
    pub fn new<T: Read>(rdr: &mut T) -> Result<(Handle<ZMemory>, ZHeader)> {
        let mut byte_vec = Vec::<u8>::new();
        rdr.read_to_end(&mut byte_vec)?;

        // Have to bootstrap these.
        let static_base =
            bytes::word_from_slice(&byte_vec, usize::from(header::HOF_STATIC_MEMORY_BASE));
        let high_base =
            bytes::word_from_slice(&byte_vec, usize::from(header::HOF_HIGH_MEMORY_BASE));

        let zmem = new_handle(ZMemory {
            bytes: byte_vec.into(),
            static_mem: ByteAddress::from_raw(static_base).into(),
            high_mem: ByteAddress::from_raw(high_base).into(),
        });

        let header = ZHeader::new(&zmem)?;

        assert_eq!(zmem.borrow().static_mem, header.static_memory_base().into());
        assert_eq!(zmem.borrow().high_mem, header.high_memory_base().into());

        Ok((zmem, header))
    }

    // The total number of bytes in the ZMemory.
    pub fn memory_size(&self) -> usize {
        self.bytes.len()
    }
}

impl Memory for ZMemory {
    fn read_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy,
    {
        self.bytes[at.into().value()]
    }

    fn write_byte<T>(&mut self, at: T, val: u8) -> Result<()>
    where
        T: Into<ZOffset> + Copy,
    {
        let offset = at.into();
        if offset < self.static_mem {
            self.bytes[offset.value()] = val;
            Ok(())
        } else {
            Err(ZErr::WriteViolation(offset.value()))
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::super::addressing::{ByteAddress, WordAddress};
    use super::super::handle::Handle;
    use super::super::version::ZVersion;
    use super::*;

    fn sample_bytes() -> Vec<u8> {
        let mut bytes = vec![
            3, // version number
            0, 0, 0, // 0x01-0x03
            0x00, 0xa0, // start of high memory
            0x00, 0x00, // start pc
            0x00, 0x00, 0x00, 0x00, 0x12, 0x34, // 0x08 - 0x0d
            0x00, 0x80, // start of static memory
        ];
        bytes.resize(0x0100, 0);
        bytes
    }

    fn make_test_mem(vers: ZVersion) -> Handle<ZMemory> {
        let mut bytes = sample_bytes();
        bytes[0] = vers as u8;
        ZMemory::new(&mut Cursor::new(bytes)).unwrap().0
    }

    #[test]
    fn test_new() {
        let zmem = make_test_mem(ZVersion::V3);

        // Check some stuff is consistent.
        // - header consistency

        // We read the entire array.
        assert_eq!(sample_bytes().len(), zmem.borrow().memory_size());
    }

    #[test]
    fn test_byte_address() {
        let zmem = make_test_mem(ZVersion::V3);

        assert_eq!(3, zmem.borrow().read_byte(ByteAddress::from_raw(0)));
        assert_eq!(0xa0, zmem.borrow().read_byte(ByteAddress::from_raw(5)));

        assert_eq!(0x0300, zmem.borrow().read_word(ByteAddress::from_raw(0)));
        assert_eq!(0x1234, zmem.borrow().read_word(ByteAddress::from_raw(0x0c)));

        // Read a word from a non-word-aligned location.
        assert_eq!(0x8000, zmem.borrow().read_word(ByteAddress::from_raw(0x0f)));
    }

    #[test]
    fn test_word_address() {
        let zmem = make_test_mem(ZVersion::V3);

        let wa = WordAddress::from_raw(0x02);
        assert_eq!(0x00a0, zmem.borrow().read_word(wa));
        zmem.borrow_mut().write_word(wa, 0x1234).unwrap();
        assert_eq!(0x1234, zmem.borrow().read_word(wa));

        // Read/write from/to a non-word-aligned location.
        let wa = WordAddress::from_raw(0x03);
        assert_eq!(0x0000, zmem.borrow().read_word(wa));
        zmem.borrow_mut().write_word(wa, 0x6789).unwrap();
        assert_eq!(0x6789, zmem.borrow().read_word(wa));
    }

    #[test]
    fn test_write_violation() {
        let zmem = make_test_mem(ZVersion::V3);

        let static_base = zmem.borrow().static_mem;
        assert!(match zmem.borrow_mut().write_byte(static_base, 0x8888) {
            Err(ZErr::WriteViolation(val)) if val == static_base.value() => true,
            _ => false,
        });
        assert!(match zmem.borrow_mut().write_word(static_base, 0x9999) {
            Err(ZErr::WriteViolation(val)) if val == static_base.value() => true,
            _ => false,
        })
    }
}
