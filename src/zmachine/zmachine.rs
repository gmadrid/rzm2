use std::io::Read;
use std::rc::Rc;

use super::result::Result;
use super::handle::Handle;
use super::header::ZHeader;
use super::memory::ZStory;

pub struct ZMachine {
    pub story: Handle<ZStory>,
    pub header: ZHeader,
}

impl ZMachine {
    pub fn new<T: Read>(rdr: &mut T) -> Result<ZMachine> {
        // TODO: error handling. get rid of unwraps.
        let story = Rc::new(ZStory::new(rdr).unwrap());
        let header = ZHeader::new(&story)?;
        Ok(ZMachine {
            story,
            header,
        })
    }
}

#[cfg(test)]
mod test {

}
