use super::addressing::{ByteAddress, ZOffset};
use super::handle::Handle;
use super::result::{Result, ZErr};
use super::traits::{Header, Memory};
use super::version::ZVersion;

// jin a b           - jump if a in b (if parent of a is b)
// test_attr o a     - jump if object has attr
// set_attr o a      - make object have attr
// clear_attr o a    - make object NOT have attr
// insert_obj o par  - make object become first child of par
// get_prop o p      - read property from object
// get_prop_addr o p - get byte address of property data of property in object
// get_next_prop o p - get the number of the next property for object
// get_sibling o     - get the next sibling of object
// get_child o       - get the first child of object
// get_parent o      - get the parent of the object
// get_prop_len o p  - get the length of the prop data for property in object
// remove_obj o      - make the parent of object be 0
// print_obj o       - print the short name of object
// put_prop o p v    - write value to property in object

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Object {
    Null,
    Address(ByteAddress),
}

impl Object {
    fn with_address<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(ByteAddress) -> Result<T>,
    {
        // TODO: range check
        match self {
            Object::Null => Err(ZErr::NullObject),
            Object::Address(ba) => f(*ba),
        }
    }

    fn offset(&self, offset: u16) -> Result<ByteAddress> {
        self.with_address(|ba| Ok(ba.inc_by(offset)))
    }
}

impl From<Object> for ZOffset {
    fn from(obj: Object) -> ZOffset {
        match obj {
            Object::Null => panic!("Taking address of Null object."),
            Object::Address(ba) => ba.into(),
        }
    }
}

pub trait ObjectTable {
    // TODO: range check.
    //    fn get_object(&self, num: u16);

    fn get_object_child(&self, o: Object) -> Result<Object>;
    fn get_object_sibling(&self, o: Object) -> Result<Object>;
    fn get_object_parent(&self, o: Object) -> Result<Object>;

    fn set_object_child(&self, o: Object, c: Object) -> Result<()>;
    fn set_object_sibling(&self, o: Object, s: Object) -> Result<()>;
    fn set_object_parent(&self, o: Object, p: Object) -> Result<()>;

    fn get_object_attribute(&self, o: Object, a: u16) -> Result<u16>;
    fn set_object_attribute(&self, o: Object, a: u16, v: u16) -> Result<()>;

    fn get_object_property(&self, o: Object, p: u8) -> Result<u16>; // Is this right? Are all properties u16?
    fn set_object_property(&self, o: Object, p: u8, v: u16) -> Result<()>;

    fn get_default_property(&self, p: u8) -> Result<u16>; // Is this right? Are all properties u16?
}

pub struct ZObjectTable<M>
where
    M: Memory,
{
    memory: Handle<M>,

    version: ZVersion,
    defaults_offset: ByteAddress,
    tree_offset: ByteAddress,
}

impl<M> ZObjectTable<M>
where
    M: Memory,
{
    fn new<H>(header: &H, memory: &Handle<M>) -> ZObjectTable<M>
    where
        H: Header,
    {
        let base = header.otable_location();
        // This depends on version number!!!!  VNUM_DEPEND
        let tree = base.inc_by(31 * 2); // 31 words in V1-3 only. Fix for V4+.
        ZObjectTable {
            memory: memory.clone(),
            version: header.version_number(),

            defaults_offset: base,
            tree_offset: tree,
        }
    }

    fn get_object(&self, num: u16) -> Object {
        // TODO: range check
        // VNUM DEPEND
        // Objects are 1-indexed. (Zero is the null object.)
        if num == 0 {
            Object::Null
        } else {
            Object::Address(self.tree_offset.inc_by((num - 1) * 9))
        }
    }
}

impl<M> ObjectTable for ZObjectTable<M>
where
    M: Memory,
{
    fn get_object_child(&self, o: Object) -> Result<Object> {
        let offset = o.offset(6);
        Ok(Object::Null)
    }

    fn get_object_sibling(&self, o: Object) -> Result<Object> {
        panic!("Unimplemented")
    }
    fn get_object_parent(&self, o: Object) -> Result<Object> {
        panic!("Unimplemented")
    }

    fn set_object_child(&self, o: Object, c: Object) -> Result<()> {
        panic!("Unimplemented")
    }
    fn set_object_sibling(&self, o: Object, s: Object) -> Result<()> {
        panic!("Unimplemented")
    }
    fn set_object_parent(&self, o: Object, p: Object) -> Result<()> {
        panic!("Unimplemented")
    }

    fn get_object_attribute(&self, o: Object, a: u16) -> Result<u16> {
        panic!("Unimplemented")
    }
    fn set_object_attribute(&self, o: Object, a: u16, v: u16) -> Result<()> {
        panic!("Unimplemented")
    }

    fn get_object_property(&self, o: Object, p: u8) -> Result<u16> // Is this right? Are all properties u16?
    {
        panic!("Unimplemented")
    }
    fn set_object_property(&self, o: Object, p: u8, v: u16) -> Result<()> {
        panic!("Unimplemented")
    }

    fn get_default_property(&self, p: u8) -> Result<u16> // Is this right? Are all properties u16?
    {
        panic!("Unimplemented")
    }
}
