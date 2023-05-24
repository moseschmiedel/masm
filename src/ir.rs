use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

pub struct IR {
    pub start_label: Label,
    pub instructions: HashMap<Label, Vec<Instruction>>,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub address: MemoryAddress,
}

impl Label {
    pub fn new(name: impl Into<String>, address: u16) -> Label {
        Label {
            name: name.into(),
            address: MemoryAddress(address),
        }
    }
}

impl Hash for Label {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Label {}

#[derive(Debug)]
pub enum Instruction {
    EmptyLine,
    Move(UnaryExpression),
    Load {
        address: RegisterAddress,
        source: LoadSource,
    },
    StoreRAM,
    Halt,
    Noop,
    Jump {
        condition: JumpCondition,
        negate: bool,
    },
    Add(BinaryExpression),
    AddWithCarry(BinaryExpression),
    Subtract(BinaryExpression),
    SubtractWithCarry(BinaryExpression),
    Increment(UnaryExpression),
    Decrement(UnaryExpression),
    Multiply(BinaryExpression),
    Test(BinaryStatement),
    AND(BinaryExpression),
    OR(BinaryExpression),
    NOT(UnaryExpression),
    XOR(BinaryExpression),
    XNOR(BinaryExpression),
    ShiftLeft(BinaryExpression),
    ShiftRight(BinaryExpression),
    Negate(UnaryExpression),
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterAddress(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryAddress(pub u16);
#[derive(Debug, Clone, Copy)]
pub struct Constant(pub u16);

#[derive(Debug, Clone, Copy)]
enum Source {
    Register(RegisterAddress),
    Memory(MemoryAddress),
    Constant(Constant),
}

#[derive(Debug, Clone, Copy)]
pub struct Register {
    address: RegisterAddress,
}

impl Register {
    pub fn new(address: RegisterAddress) -> Self {
        Self { address }
    }

    pub fn addr(&self) -> u8 {
        self.address.0
    }
}

#[derive(Debug)]
pub struct UnaryExpression {
    target: Register,
    source_a: Register,
}

#[derive(Debug)]
pub struct BinaryExpression {
    pub target: Register,
    pub source_a: Register,
    pub source_b: Register,
}

impl BinaryExpression {
    pub fn new(target: Register, source_a: Register, source_b: Register) -> Self {
        Self {
            target,
            source_a,
            source_b,
        }
    }
}

#[derive(Debug)]
pub struct BinaryStatement {
    source_a: Register,
    source_b: Register,
}

#[derive(Debug)]
struct TernaryExpression {
    target: Register,
    source_a: Register,
    source_b: Register,
    source_c: Register,
}

#[derive(Debug)]
pub enum LoadSource {
    Constant(u16),
    RAM { address: MemoryAddress },
    Pgm,
}

#[derive(Debug)]
pub enum JumpCondition {
    True,
    Zero,
    Less,
}
