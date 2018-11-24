extern crate rzm2;

use std::fs::File;

use rzm2::{Result, ZMachine};

fn run() -> Result<()> {
    // TODO: add some cmd line args.
    // TODO: read a filename.
    let mut rdr = File::open("Zork1.z3")?;
    let mut machine = ZMachine::new(&mut rdr)?;
    machine.run();
    println!("RAN IT");
    Ok(())
}

fn main() {
    match run() {
        Ok(_) => (),
        //        Err(ItoolsError::Clap(err)) => println!("{}", err.description()),
        Err(e) => println!("Error: {:?}", e),
    }
}
