use super::handle::Handle;
use super::memory::{ByteAddress, ZStory};
use super::result::Result;
use super::zversion::ZVersion;

// Read a Story's Header information.
// See ZSpec 11.
pub struct ZHeader {
    story: Handle<ZStory>,

    // The first byte of the story file.
    // We cache this because we use it a lot. It's read-only in the story,
    // so we don't have to worry about mutation.
    z_version: ZVersion,
}

impl ZHeader {
    pub fn new(story: &Handle<ZStory>) -> Result<ZHeader> {
        Ok(ZHeader {
            story: story.clone(),
            z_version: ZVersion::new(story.read_byte(ByteAddress::from_raw(0x00).into()))?,
        })
    }

    pub fn version_number(&self) -> ZVersion {
        self.z_version
    }

    pub fn file_length(&self) -> usize {
        let raw_file_length = self.story.read_word(ByteAddress::from_raw(0x1A).into());
        self.z_version.convert_file_length(raw_file_length)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use std::rc::Rc;

    use super::super::result::ZErr;
    
    use super::*;

    fn basic_header() -> Vec<u8> {
        vec![
            3,  // 0x00: version number (3)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // 0x01-0x07
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // 0x08 - 0x0f
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,  // 0x10 - 0x17
            0x00, 0x00,  // 0x18 - 0x19
            0x00, 0x12,  // 0x1a - 0x1b: file length
            0x00, 0x00, 0x00, 0x00,  // 0x1c - 0x1f
            0x00, 0x00, 0x00, 0x00,  // 0x20-0x23
        ]
    }

    fn new_test_story() -> Handle<ZStory> {
        new_story_from_bytes(&basic_header())
    }

    fn new_story_from_bytes(bytes: &[u8]) -> Handle<ZStory> {
        Rc::new(ZStory::new(&mut Cursor::new(bytes)).unwrap())
    }

    #[test]
    fn test_basic() {
        let story = new_test_story();
        let hdr = ZHeader::new(&story).unwrap();
        assert_eq!(ZVersion::V3, hdr.version_number());
    }

    #[test]
    fn test_file_length() {
        let story = new_test_story();
        let hdr = ZHeader::new(&story).unwrap();
        assert_eq!(0x24, hdr.file_length());

        let mut v5_bytes = basic_header();
        v5_bytes[0] = 5;
        v5_bytes[0x1b] = 0x09;
        let story = new_story_from_bytes(&v5_bytes);
        let hdr = ZHeader::new(&story).unwrap();
        assert_eq!(0x24, hdr.file_length());
    }

    #[test]
    fn test_bad_version() {
        let mut my_bytes = basic_header();
        my_bytes[0] = 0x80;
        let story = new_story_from_bytes(&my_bytes);

        match ZHeader::new(&story) {
            Err(ZErr::UnknownVersionNumber(0x80)) => (),
            _ => panic!("Something broke."),
        }
    }
}
