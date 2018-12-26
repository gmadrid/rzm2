use itertools::Itertools;

use super::addressing::{ByteAddress, WordAddress, ZOffset};
use super::handle::Handle;
use super::result::Result;
use super::traits::{Memory, PC};

// TODO: make this a struct to avoid so much param passing.

const V2_TO_4_TABLE: [char; 78] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', //
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', //
    ' ', '\n', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '.', ',', '!', '?', '_', '#',
    '\'', '"', '/', '\\', '-', ':', '(', ')',
];

// TODO: all of these ByteAddresses should be B: Into<ZOffset>
pub fn read_zstr_from_pc<M, P>(
    memory: &Handle<M>,
    abbrev_offset: ByteAddress,
    pc: &mut P,
) -> Result<String>
where
    M: Memory,
    P: PC,
{
    read_zstr(memory, abbrev_offset, || Ok(pc.next_word()))
}

pub fn read_abbrev<M>(
    mem: &Handle<M>,
    abbrev_offset: ByteAddress,
    abbrev_table: u8,
    abbrev_number: u8,
) -> Result<String>
where
    M: Memory,
{
    let entry_number = 32 * (abbrev_table - 1) + abbrev_number;
    let entry_address = abbrev_offset.inc_by(u16::from(entry_number) * 2);
    //    let entry_address = ByteAddress::from_raw(u16::from(entry_number) * 2).inc_by(abbrev_offset);
    let abbrev_address = WordAddress::from_raw(mem.borrow().read_word(entry_address));

    read_zstr_from_memory(mem, abbrev_offset, abbrev_address)
}

pub fn read_zstr_from_memory<M, O>(
    mem: &Handle<M>,
    abbrev_offset: ByteAddress,
    offset: O,
) -> Result<String>
where
    M: Memory,
    O: Into<ZOffset> + Copy,
{
    let mut zoffset = offset.into();
    read_zstr(mem, abbrev_offset, || {
        let word = mem.borrow().read_word(zoffset);
        zoffset = zoffset.inc_by(2);
        Ok(word)
    })
}

fn read_zstr<F, M>(
    memory: &Handle<M>,
    abbrev_offset: ByteAddress,
    mut next_word: F,
) -> Result<String>
where
    F: FnMut() -> Result<u16>,
    M: Memory,
{
    let mut zstr = "".to_string();
    let mut next_char_offset = 0;
    let mut abbrev_table = 0;
    loop {
        let word = next_word()?;
        let (done, bytes) = break_apart_word(word);

        for byte in bytes.iter() {
            // TODO: range check.
            let char_offset = next_char_offset;
            next_char_offset = 0;

            if abbrev_table > 0 {
                let r = zstr.push_str(&read_abbrev(memory, abbrev_offset, abbrev_table, *byte)?);
                abbrev_table = 0;
                r
            } else {
                match byte {
                    0 => zstr.push(' '),

                    // TODO: learn the pattern notation to clean this up.
                    1 => abbrev_table = 1,
                    2 => abbrev_table = 2,
                    3 => abbrev_table = 3,

                    4 => next_char_offset = 26,
                    5 => next_char_offset = 52,
                    6...31 => {
                        zstr.push(V2_TO_4_TABLE[usize::from(char_offset + byte - 6)]);
                    }
                    v => {
                        println!("V: {}", v);
                    }
                }
            }
        }

        if done {
            break;
        }
    }
    Ok(zstr)
}

fn break_apart_word(word: u16) -> (bool, [u8; 3]) {
    let done = (word & 0b1000_0000_0000_0000) != 0;
    let byte1 = (word & 0b0111_1100_0000_0000) >> 10;
    let byte2 = (word & 0b0000_0011_1110_0000) >> 5;
    let byte3 = word & 0b0000_0000_0001_1111;

    (done, [byte1 as u8, byte2 as u8, byte3 as u8])
}

// Assemble three "u5"s into a Z-string encoded u16. The top bit will indicate whether
// this is the last word in the string, as specified in the ZSpec.
//
// All input chars will be truncated to 5-bits.
fn assemble_word(end_of_string: bool, chars: [u8; 3]) -> u16 {
    let done_bit = if end_of_string { 0b1000_0000_0000_0000u16 } else { 0 };
    let piece1 = (u16::from(chars[0]) & 0b11111) << 10;
    let piece2 = (u16::from(chars[1]) & 0b11111) << 5;
    let piece3 = (u16::from(chars[2]) & 0b11111);

    done_bit | piece1 | piece2 | piece3
}


#[cfg(test)]
mod tests {
    use super::*;

    use super::super::handle::new_handle;
    use super::super::fixtures::{ TestMemory, TestPC} ;

    struct ZChars<I> {
        iter: I,
        waiting: Option<u8>,
    }

    fn zchars<I: Iterator<Item=char>>(i: I) -> ZChars<I> {
        ZChars { iter: i, waiting: None }
    }

    impl<I: Iterator<Item=char>> Iterator for ZChars<I> {
        type Item = u8;

        fn next(&mut self) -> Option<Self::Item> {
            self.iter.next().map(|ch| {
                match ch {
                    'a'...'z' => { (ch as u8) - ('a' as u8) + 6 }
                    _ => unimplemented!("conversion for char '{}' unimplemented", ch),
                }
            })
        }
    }

    fn encode_simple_string(st: &str) -> Vec<u8> {
        let mut encoded = vec![];

        let foo = zchars(st.chars());
        for zc in foo {

        }
        //for zc in ZChars{ iter: st.chars() }
//        st.chars().map();
//        for chunk in &st.chars().chunks(3) {
//            let mut chs = [5u8, 5, 5];
//            for (i, ch) in chunk.enumerate() {
//                chs[i] = ch as u8;
//            }
//            let word = assemble_word(false, chs);
//            encoded.push((word >> 8) as u8);
//            encoded.push((word & 0xff) as u8);
//        }
//        let end_indicator_position = encoded.len() - 2;
//        encoded[end_indicator_position] = encoded[end_indicator_position] | 0x80;
//
        encoded
    }

    #[test]
    fn test_assemble_word() {
        assert_eq!(0b0_11111_00000_11011, assemble_word(false, [0xff, 0xe0, 0x1b]));
        assert_eq!(0b1_00101_10010_00100, assemble_word(true, [5, 18, 4]));
    }

    #[test]
    fn test_encode_string() {
        let mem = new_handle(TestMemory::new(0));
        let mut pc = TestPC::new(0, encode_simple_string("blasphemy"));
        eprintln!("pc = {:#?}", pc.values);
        for val in &pc.values {
            eprintln!("   = {:08b}", val);
        }
        let foo = read_zstr_from_pc(&mem, ByteAddress::from_raw(0), &mut pc);
        eprintln!("foo = {:?}", foo.unwrap());
            assert_eq!(vec![32u8, 2], encode_simple_string("Hi there"))
    }
}