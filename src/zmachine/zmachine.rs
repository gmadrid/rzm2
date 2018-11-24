use std::io::Read;

use super::addressing::PC;
use super::handle::Handle;
use super::header::ZHeader;
use super::memory::ZMemory;
use super::result::Result;
use super::stack::ZStack;

pub struct ZMachine {
    pub story_h: Handle<ZMemory>,
    pub header: ZHeader,
    pub pc: PC,
    pub stack: ZStack,
}

impl ZMachine {
    pub fn new<T: Read>(rdr: &mut T) -> Result<ZMachine> {
        // TODO: error handling. get rid of unwraps.
        let (story_h, header) = ZMemory::new(rdr)?;
        let pc = PC::new(header.start_pc(), header.version_number());
        let stack = ZStack::new();

        Ok(ZMachine {
            story_h,
            header,
            pc,
            stack,
        })
    }
}

#[cfg(test)]
mod test {}
