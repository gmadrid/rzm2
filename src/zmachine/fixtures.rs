use std::collections::HashMap;

use super::addressing::ZOffset;
use super::opcode::ZVariable;
use super::traits::{Memory, Variables, PC};

pub struct TestPC {
    pub pc: usize,
    pub values: Vec<u8>,
}

impl TestPC {
    pub fn new(pc: usize, values: Vec<u8>) -> TestPC {
        TestPC { pc, values }
    }
}

impl PC for TestPC {
    fn current_pc(&self) -> usize {
        self.pc
    }

    fn next_byte(&mut self) -> u8 {
        self.pc += 1;
        self.values.remove(0)
    }
}

#[derive(Default)]
pub struct TestVariables {
    pub variables: HashMap<ZVariable, u16>,
}

impl TestVariables {
    pub fn new() -> TestVariables {
        TestVariables::default()
    }
}

impl Variables for TestVariables {
    fn read_variable(&self, var: ZVariable) -> u16 {
        *self.variables.get(&var).unwrap_or(&0)
    }

    fn write_variable(&mut self, var: ZVariable, val: u16) {
        self.variables.insert(var, val);
    }
}

pub struct TestMemory {
    pub bytes: Vec<u8>,
}

impl TestMemory {
    pub fn new(size: usize) -> TestMemory {
        let vec = vec![0; size];
        TestMemory { bytes: vec }
    }
}

impl Memory for TestMemory {
    fn get_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy,
    {
        panic!("unimplemented")
    }

    fn set_byte<T>(&mut self, at: T, val: u8)
    where
        T: Into<ZOffset> + Copy,
    {
        let offset = at.into();
        self.bytes[offset.value()] = val;
    }
}
