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
    Move(UnaryExpression),
    Load {
        address: RegisterAddress,
        source: LoadSource,
    },
    StoreRAM {
        address_register: RegisterAddress,
        data_register: RegisterAddress,
    },
    Halt,
    Noop,
    Jump {
        target: JumpTarget,
        condition: JumpCondition,
    },
    Add(BinaryExpression),
    Add3(TernaryExpression),
    AddWithCarry(BinaryExpression),
    Subtract(BinaryExpression),
    SubtractWithCarry(BinaryExpression),
    Increment(UnaryStatement),
    Decrement(UnaryStatement),
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
pub struct Register {
    pub address: RegisterAddress,
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
    pub target: Register,
    pub source_a: Register,
}

impl UnaryExpression {
    pub fn new(target: Register, source_a: Register) -> Self {
        Self { target, source_a }
    }
}

#[derive(Debug)]
pub struct UnaryStatement {
    pub source_a: Register,
}

impl UnaryStatement {
    pub fn new(source_a: Register) -> Self {
        Self { source_a }
    }
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
    pub source_a: Register,
    pub source_b: Register,
}

impl BinaryStatement {
    pub fn new(source_a: Register, source_b: Register) -> Self {
        Self { source_a, source_b }
    }
}

#[derive(Debug)]
pub struct TernaryExpression {
    pub target: Register,
    pub source_a: Register,
    pub source_b: Register,
    pub source_c: Register,
}

impl TernaryExpression {
    pub fn new(
        target: Register,
        source_a: Register,
        source_b: Register,
        source_c: Register,
    ) -> TernaryExpression {
        TernaryExpression {
            target,
            source_a,
            source_b,
            source_c,
        }
    }
}

#[derive(Debug)]
pub enum LoadSource {
    Constant(u16),
    RAM { address_register: Register },
    Pgm,
}

#[derive(Debug)]
pub enum JumpTarget {
    Constant(u16),
    Register(Register),
    Label(Label),
}

#[derive(Debug)]
pub enum JumpCondition {
    True,
    Zero,
    NotZero,
    Less,
}
