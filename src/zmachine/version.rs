use super::result::{Result, ZErr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZVersion {
    //    V1 = 1,
    //    V2 = 2,
    V3 = 3,
    //    V4 = 4,
    V5 = 5,
    //    V6 = 6,
}

impl ZVersion {
    pub fn new(byte: u8) -> Result<ZVersion> {
        use self::ZVersion::*;
        match byte {
            //            1 => Ok(V1),
            //            2 => Ok(V2),
            3 => Ok(V3),
            //            4 => Ok(V4),
            5 => Ok(V5),
            //            6 => Ok(V6),
            _ => Err(ZErr::UnknownVersionNumber(byte)),
        }
    }

    pub fn convert_file_length(&self, raw_length: u16) -> usize {
        use self::ZVersion::*;
        (match self {
            //            V1 | V2 |
            V3 => 2,
            //            V4 |
            V5 => 4,
            //            V6 => 8,
        }) as usize
            * raw_length as usize
    }
}
