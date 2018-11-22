use std::io::Read;

use super::handle::Handle;
use super::result::Result;

// Locations in the story are addressed using ZOffsets. The ZOffset is an index
// into story memory. The ZMachine uses three types of addresses to refer to
// memory. Each of these types maps to a ZOffset in a different way, possibly even
// different depending on the story's ZMachine version number.
#[derive(Clone, Copy, Debug)]
pub struct Offset(usize);

#[derive(Clone, Copy, Debug)]
pub struct ByteAddress(u16);

impl ByteAddress {
    pub fn from_raw(word: u16) -> ByteAddress {
        ByteAddress(word)
    }
}

impl From<ByteAddress> for Offset {
    fn from(ba: ByteAddress) -> Offset {
        Offset(ba.0 as usize)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WordAddress(u16);

#[derive(Clone, Copy)]
pub struct PackedAddress(u16);
    

// A representation of a loaded story file.
// The story file is memory-mapped into the ZMachine's "core memory", and the
// ZStory controls access to this memory enforcing 1) read-only vs read/write
// memory locations, mapping different types of addresses according to the
// story version.
pub struct ZStory {
    bytes: Handle<[u8]>,
}

impl ZStory {
    // Read story from rdr.
    //
    // Consumes the entire contents of the rdr.
    //
    // May fail if rdr returns an error or if the mapped memory
    // fails consistency checks.
    pub fn new<T: Read>(rdr: &mut T) -> Result<ZStory> {
        let mut byte_vec = Vec::<u8>::new();
        rdr.read_to_end(&mut byte_vec)?;

        Ok(ZStory {
            bytes: byte_vec.into(),
        })
    }

    pub fn read_byte(&self, index: Offset) -> u8 {
        self.bytes[index.0]
    }

    pub fn read_word(&self, index: Offset) -> u16 {
        let high_byte = self.bytes[index.0];
        let low_byte = self.bytes[index.0 + 1];
        ((high_byte as u16) << 8) + (low_byte as u16)
    }

    // The length of the story file.
    #[cfg(test)]
    fn story_len(&self) -> usize {
        self.bytes.len()
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::*;

    const BYTES: &[u8] = &[3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 0xcc, 0xdd];

    #[test]
    fn test_new() {
        let zmem = ZStory::new(&mut Cursor::new(BYTES)).unwrap();

        // Check some stuff is consistent.
        // - header consistency

        // We read the entire array.
        assert_eq!(BYTES.len(), zmem.story_len());
    }

    #[test]
    fn test_read_bytes() {
        let zmem = ZStory::new(&mut Cursor::new(BYTES)).unwrap();

        assert_eq!(3, zmem.read_byte(Offset(0)));
        assert_eq!(8, zmem.read_byte(Offset(5)));

        assert_eq!(0x0304, zmem.read_word(Offset(0)));
        assert_eq!(0xccdd, zmem.read_word(Offset(0x0a)));

        // Try reading a word from non-word-aligned location.
        assert_eq!(0x09cc, zmem.read_word(Offset(0x09)));
    }
}
