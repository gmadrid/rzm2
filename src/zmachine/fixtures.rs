use std::collections::HashMap;

use super::addressing::ZOffset;
use super::opcode::ZVariable;
use super::result::{Result, ZErr};
use super::traits::{Memory, Stack, Variables, PC};

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

    fn set_current_pc(&mut self, new_pc: usize) {
        self.pc = new_pc;
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
    fn read_variable(&mut self, var: ZVariable) -> Result<u16> {
        self.variables
            .get(&var)
            .map(|v| *v)
            .ok_or(ZErr::GenericError("Variable missing"))
    }

    fn write_variable(&mut self, var: ZVariable, val: u16) -> Result<()> {
        self.variables.insert(var, val);
        Ok(())
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

    pub fn new_from_vec(bytes: Vec<u8>) -> TestMemory {
        TestMemory { bytes }
    }
}

impl Memory for TestMemory {
    fn read_byte<T>(&self, at: T) -> u8
    where
        T: Into<ZOffset> + Copy,
    {
        self.bytes[at.into().value()]
    }

    fn write_byte<T>(&mut self, at: T, val: u8) -> Result<()>
    where
        T: Into<ZOffset> + Copy,
    {
        let offset = at.into();
        self.bytes[offset.value()] = val;
        Ok(())
    }
}

#[derive(Default)]
pub struct TestStack {
    pub arr: Vec<u8>, // a very small stack.
    pub map: HashMap<u8, u16>,
}

impl TestStack {
    pub fn new(size: usize) -> TestStack {
        TestStack {
            arr: vec![0; size],
            ..TestStack::default()
        }
    }
}

impl Stack for TestStack {
    fn push_byte(&mut self, val: u8) -> Result<()> {
        self.arr.push(val);
        Ok(())
    }
    fn pop_byte(&mut self) -> Result<u8> {
        self.arr
            .pop()
            .ok_or(ZErr::StackUnderflow("Underflow in TestStack"))
    }

    fn read_local(&self, l: u8) -> Result<u16> {
        let value = if self.map.contains_key(&l) {
            self.map[&l]
        } else {
            0
        };
        Ok(value)
    }

    fn write_local(&mut self, l: u8, val: u16) -> Result<()> {
        self.map.insert(l, val);
        Ok(())
    }

    fn push_frame(
        &mut self,
        _return_pc: usize,
        _num_locals: u8,
        _return_var: ZVariable,
        _operands: &[u16],
    ) -> Result<()> {
        panic!("unimplemented");
    }

    fn pop_frame(&mut self) -> Result<()> {
        Ok(())
    }
    fn return_pc(&self) -> usize {
        panic!("unimplemented")
    }
    fn return_variable(&self) -> ZVariable {
        panic!("unimplemented")
    }
}
