use super::addressing::ZOffset;
use super::opcode::ZVariable;
use super::version::ZVersion;

pub trait Header {
    fn version_number(&self) -> ZVersion;
}

pub trait PC {
    fn current_pc(&self) -> usize;
    fn next_byte(&mut self) -> u8;

    fn next_word(&mut self) -> u16 {
        let high_byte = self.next_byte();
        let low_byte = self.next_byte();
        (u16::from(high_byte) << 8) + u16::from(low_byte)
    }
}

pub trait Memory {
    fn set_byte<T>(&mut self, at: T, val: u8)
    where
        T: Into<ZOffset>;

    fn set_word<T>(&mut self, at: T, val: u16)
    where
        T: Into<ZOffset> + Clone,
    {
        let high_byte = (val >> 8) as u8;
        let low_byte = (val & 0xff) as u8;
        let offset = at.into();
        self.set_byte(offset, high_byte);
        self.set_byte(offset.inc_by(1), low_byte);
    }
}

pub trait Stack {
    fn push_byte(&mut self, val: u8);
    fn pop_byte(&mut self) -> u8;

    fn push_word(&mut self, word: u16) {
        self.push_byte((word >> 8 & 0xff) as u8);
        self.push_byte((word >> 0 & 0xff) as u8);
    }

    fn pop_word(&mut self) -> u16 {
        let low_byte = u16::from(self.pop_byte());
        let high_byte = u16::from(self.pop_byte());

        (high_byte << 8) + low_byte
    }
}

pub trait Variables {
    fn read_variable(&self, var: ZVariable) -> u16;
    fn write_variable(&mut self, var: ZVariable, val: u16);
}
