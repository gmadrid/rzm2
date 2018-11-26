#[macro_use]
extern crate log;

mod zmachine;

pub use zmachine::new_story_processor;
pub use zmachine::Result;
pub use zmachine::ZProcessor;
