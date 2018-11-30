// UNREVIEWED

use std::cell::RefCell;
use std::rc::Rc;

pub type Handle<T> = Rc<RefCell<T>>;

pub fn new_handle<T>(t: T) -> Handle<T> {
    Rc::new(RefCell::new(t))
}
