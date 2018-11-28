use std::io::Read;

use super::addressing::ZOffset;
use super::handle::{new_handle, Handle};
use super::header::ZHeader;
use super::result::Result;
use super::traits::Memory;

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

        let zmem = new_handle(ZMemory {
            bytes: byte_vec.into(),
        });

        let header = ZHeader::new(&zmem)?;

        Ok((zmem, header))
    }

    // The total number of bytes in the ZMemory.
    pub fn memory_size(&self) -> usize {
        self.bytes.len()
    }

    // Read the byte at location ZOffset in the ZMemory.
    pub fn read_byte<T>(&self, index: T) -> u8
    where
        T: Into<ZOffset>,
    {
        self.bytes[index.into().value()]
    }

    // Read the (big-endian) word at location ZOffset in ZMemory.
    // The ZOffset need not be word-aligned.
    pub fn read_word<T>(&self, index: T) -> u16
    where
        T: Into<ZOffset>,
    {
        let offset = index.into();
        let high_byte = self.bytes[offset.value()];
        let low_byte = self.bytes[offset.value() + 1];
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

impl Memory for ZMemory {
    fn get_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy,
    {
        self.bytes[at.into().value()]
    }

    fn set_byte<T>(&mut self, at: T, val: u8)
    where
        T: Into<ZOffset> + Copy,
    {
        self.bytes[at.into().value()] = val;
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::super::addressing::ByteAddress;
    use super::super::handle::Handle;
    use super::super::version::ZVersion;
    use super::*;

    fn sample_bytes() -> Vec<u8> {
        vec![3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 0xcc, 0xdd]
    }

    fn make_test_mem(vers: ZVersion) -> Handle<ZMemory> {
        let mut bytes = sample_bytes();
        bytes[0] = vers as u8;
        ZMemory::new(&mut Cursor::new(&bytes)).unwrap().0
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
        assert_eq!(8, zmem.borrow().read_byte(ByteAddress::from_raw(5)));

        assert_eq!(0x0304, zmem.borrow().read_word(ByteAddress::from_raw(0)));
        assert_eq!(0xccdd, zmem.borrow().read_word(ByteAddress::from_raw(0x0a)));

        // Read a word from a non-word-aligned location.
        assert_eq!(0x09cc, zmem.borrow().read_word(ByteAddress::from_raw(0x09)));
    }
}
