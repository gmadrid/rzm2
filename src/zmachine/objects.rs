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

pub struct ObjectNumber(u16);

pub trait Object {}

pub trait ObjectTable {
    type O: Object;

    // TODO: range check.
    fn get_object(&self, num: ObjectNumber) -> Result<Self::O>;

    fn get_object_child(&self, o: Self::O) -> Result<ObjectNumber>;
    fn get_object_sibling(&self, o: Self::O) -> Result<ObjectNumber>;
    fn get_object_parent(&self, o: Self::O) -> Result<ObjectNumber>;

    fn set_object_child(&self, o: Self::O, c: ObjectNumber) -> Result<()>;
    fn set_object_sibling(&self, o: Self::O, s: ObjectNumber) -> Result<()>;
    fn set_object_parent(&self, o: Self::O, p: ObjectNumber) -> Result<()>;

    fn get_object_attribute(&self, o: Self::O, a: u8) -> Result<u8>;
    fn set_object_attribute(&self, o: Self::O, a: u8, v: u8) -> Result<()>;

    fn get_object_property(&self, o: Self::O, p: u8) -> Result<u16>; // Is this right? Are all properties u16?
    fn set_object_property(&self, o: Self::O, p: u8, v: u16) -> Result<()>;

    fn get_default_property(&self, p: u8) -> Result<u16>; // Is this right? Are all properties u16?
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZObject(ByteAddress);

impl Object for ZObject {}

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
}

impl<M> ObjectTable for ZObjectTable<M>
where
    M: Memory,
{
    type O = ZObject;

    fn get_object(&self, num: ObjectNumber) -> Result<ZObject> {
        // TODO: range check
        // VNUM DEPEND
        // Objects are 1-indexed. (Zero is the null object.)
        if num.0 == 0 {
            Err(ZErr::NullObject)
        } else {
            Ok(ZObject(self.tree_offset.inc_by((num.0 - 1) * 9)))
        }
    }

    // Consider returning Option here instead of an ObjectNumber(0).
    fn get_object_child(&self, o: ZObject) -> Result<ObjectNumber> {
        // VNUM DEPEND
        let raw_number = self.memory.borrow().read_byte(ZOffset::from(o.0.inc_by(6)));
        Ok(ObjectNumber(u16::from(raw_number)))
    }

    fn get_object_sibling(&self, o: ZObject) -> Result<ObjectNumber> {
        // VNUM DEPEND
        let raw_number = self.memory.borrow().read_byte(ZOffset::from(o.0.inc_by(5)));
        Ok(ObjectNumber(u16::from(raw_number)))
    }
    fn get_object_parent(&self, o: ZObject) -> Result<ObjectNumber> {
        // VNUM DEPEND
        let raw_number = self.memory.borrow().read_byte(ZOffset::from(o.0.inc_by(4)));
        Ok(ObjectNumber(u16::from(raw_number)))
    }

    fn set_object_child(&self, o: ZObject, c: ObjectNumber) -> Result<()> {
        // VNUM DEPEND
        self.memory.borrow_mut().write_byte(o.0.inc_by(6), c.0 as u8)
    }
    fn set_object_sibling(&self, o: ZObject, s: ObjectNumber) -> Result<()> {
        // VNUM DEPEND
        self.memory.borrow_mut().write_byte(o.0.inc_by(5), s.0 as u8)
    }
    fn set_object_parent(&self, o: ZObject, p: ObjectNumber) -> Result<()> {
        // VNUM DEPEND
        self.memory.borrow_mut().write_byte(o.0.inc_by(4), p.0 as u8)
    }

    fn get_object_attribute(&self, o: ZObject, a: u8) -> Result<u8> {
        // VNUM DEPEND
        // range check.
        let ba = o.0.inc_by(if a > 15 { 1} else { 0} );
        let bitnum = a % 16;
        let word = self.memory.borrow().read_word(ba);
        Ok(((word >> (15 - bitnum)) & 0b1) as u8)
    }
    
    fn set_object_attribute(&self, o: ZObject, a: u8, v: u8) -> Result<()> {
        // VNUM DEPEND
        // range check
        let ba = o.0.inc_by(if a > 15 { 1 } else { 0 });
        let word = self.memory.borrow().read_word(ba);
        let bitnum = a % 16;
        let the_bit = 1 << (15 - bitnum);
        let new_word = if v == 0 {
            word & !the_bit
        } else {
            word | the_bit
        };

        self.memory.borrow_mut().write_word(ba, new_word)
    }

    fn get_object_property(&self, o: ZObject, p: u8) -> Result<u16> // Is this right? Are all properties u16?
    {
        panic!("Unimplemented")
    }

    fn set_object_property(&self, o: ZObject, p: u8, v: u16) -> Result<()> {
        panic!("Unimplemented")
    }

    fn get_default_property(&self, p: u8) -> Result<u16> // Is this right? Are all properties u16?
    {
        panic!("Unimplemented")
    }
}
