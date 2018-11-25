use super::addressing::ByteAddress;
use super::handle::Handle;
use super::memory::ZMemory;
use super::result::Result;
use super::version::ZVersion;

// Offsets for fields in the header.
pub const OS_VERSION: u16 = 0x00;
pub const OS_START_PC: u16 = 0x06;
pub const OS_FILE_LEN: u16 = 0x1a;

// Read a Story's Header information.
// See ZSpec 11.
pub struct ZHeader {
    memory: Handle<ZMemory>,

    // The first byte of the memory file.
    // We cache this because we use it a lot. It's read-only in the memory,
    // so we don't have to worry about mutation.
    z_version: ZVersion,
}

impl ZHeader {
    pub fn new(memory: &Handle<ZMemory>) -> Result<ZHeader> {
        let z_version = ZVersion::new(memory.read_byte(ByteAddress::from_raw(OS_VERSION)))?;

        Ok(ZHeader {
            memory: memory.clone(),
            z_version,
        })
    }

    pub fn version_number(&self) -> ZVersion {
        self.z_version
    }

    pub fn start_pc(&self) -> ByteAddress {
        let raw_value = self.memory.read_word(ByteAddress::from_raw(OS_START_PC));
        ByteAddress::from_raw(raw_value)
    }

    pub fn file_length(&self) -> usize {
        let raw_file_length = self.memory.read_word(ByteAddress::from_raw(OS_FILE_LEN));
        self.z_version.convert_file_length(raw_file_length)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::super::result::ZErr;

    use super::*;

    fn basic_header() -> Vec<u8> {
        vec![
            3, // 0x00: version number (3)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x01-0x07
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x08 - 0x0f
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // 0x10 - 0x17
            0x00, 0x00, // 0x18 - 0x19
            0x00, 0x12, // 0x1a - 0x1b: file length
            0x00, 0x00, 0x00, 0x00, // 0x1c - 0x1f
            0x00, 0x00, 0x00, 0x00, // 0x20-0x23
        ]
    }

    fn new_test_story() -> (Handle<ZMemory>, ZHeader) {
        new_story_from_bytes(&basic_header()).unwrap()
    }

    fn new_story_from_bytes(bytes: &[u8]) -> Result<(Handle<ZMemory>, ZHeader)> {
        ZMemory::new(&mut Cursor::new(bytes))
    }

    #[test]
    fn test_basic() {
        let (_, hdr) = new_test_story();
        assert_eq!(ZVersion::V3, hdr.version_number());
    }

    #[test]
    fn test_file_length() {
        let (_, hdr) = new_test_story();
        assert_eq!(0x24, hdr.file_length());

        let mut v5_bytes = basic_header();
        v5_bytes[0] = 5;
        v5_bytes[0x1b] = 0x09;
        let (_, hdr) = new_story_from_bytes(&v5_bytes).unwrap();
        assert_eq!(0x24, hdr.file_length());
    }

    #[test]
    fn test_bad_version() {
        let mut my_bytes = basic_header();
        my_bytes[0] = 0x80;
        let story = new_story_from_bytes(&my_bytes);

        match story {
            Err(ZErr::UnknownVersionNumber(0x80)) => (),
            _ => panic!("Something broke."),
        }
    }
}
