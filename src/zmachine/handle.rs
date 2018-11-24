use std::rc::Rc;

pub type Handle<T> = Rc<T>;

pub fn new_handle<T>(t: T) -> Handle<T> {
    Rc::new(t)
}
