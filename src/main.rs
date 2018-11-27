extern crate env_logger;
#[macro_use]
extern crate log;
extern crate rzm2;

use std::fs::File;

use rzm2::{new_story_processor, Result};

fn run() -> Result<()> {
    // TODO: add some cmd line args.
    // TODO: read a filename.
    let mut rdr = File::open("Zork1.z3")?;
    let mut machine = new_story_processor(&mut rdr)?;
    machine.run()
}

fn main() {
    env_logger::init();

    match run() {
        Ok(_) => (),
        //        Err(ItoolsError::Clap(err)) => println!("{}", err.description()),
        Err(e) => error!("Error: {:?}", e),
    }
}
