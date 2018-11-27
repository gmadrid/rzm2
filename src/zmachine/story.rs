use std::io::Read;

use super::addressing::ZPC;
use super::header::ZHeader;
use super::memory::ZMemory;
use super::processor::ZProcessor;
use super::result::Result;
use super::stack::ZStack;
use super::traits::Header;

pub fn new_story_processor<T: Read>(
    rdr: &mut T,
) -> Result<ZProcessor<ZHeader, ZMemory, ZPC, ZStack>> {
    let (story_h, header) = ZMemory::new(rdr)?;
    // TODO: For V6, you will need to treat the start_pc as a PackedAddress.
    let pc = ZPC::new(&story_h, header.start_pc(), header.version_number());
    let stack = ZStack::new();

    Ok(ZProcessor::new(story_h, header, pc, stack))
}
