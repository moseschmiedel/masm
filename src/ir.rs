use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

/// Main struct with the parser result. `IR` stands for
/// `intermediate representation` and consists of three components:
///     - `start_label` - reference to the label at which the execution should be started
///     - `label_definitions` - lookup-table for the labels, used for getting the address
///         of the LabelDefinition referenced by a LabelReference
///     - `instructions` - collection which stores for each label the associated instructions
///         in a `Vec`
pub struct IR {
    pub start_label: LabelReference,
    pub label_definitions: LabelLUT,
    pub instructions: HashMap<LabelReference, Vec<Instruction>>,
}

#[derive(Debug, Clone)]
pub struct LabelLUT(pub HashMap<LabelReference, LabelDefinition>);

impl LabelLUT {
    pub fn new() -> Self {
        LabelLUT(HashMap::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        LabelLUT(HashMap::with_capacity(capacity))
    }
}

impl Default for LabelLUT {
    fn default() -> Self {
        LabelLUT::new()
    }
}

#[derive(Debug, Clone)]
pub struct LabelDefinition {
    pub name: String,
    pub address: MemoryAddress,
}

#[derive(Debug, Clone)]
pub struct LabelReference(String);

impl LabelDefinition {
    pub fn new(name: impl Into<String>, address: u16) -> LabelDefinition {
        LabelDefinition {
            name: name.into(),
            address: MemoryAddress(address),
        }
    }
}

impl LabelReference {
    pub fn new(name: impl Into<String>) -> Self {
        LabelReference(name.into())
    }
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl From<LabelDefinition> for LabelReference {
    fn from(value: LabelDefinition) -> Self {
        LabelReference(value.name)
    }
}

impl Hash for LabelReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name().hash(state);
    }
}

impl PartialEq for LabelReference {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}

impl Eq for LabelReference {}

/// Enum which represents all possible instructions
/// and its metadata for the assembled language
#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Move(UnaryExpression),
    Set32BitMode {
        enable: Boolean,
    },
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterAddress(pub u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoryAddress(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Constant(pub u16);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Boolean(pub bool);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryExpression {
    pub target: Register,
    pub source_a: Register,
}

impl UnaryExpression {
    pub fn new(target: Register, source_a: Register) -> Self {
        Self { target, source_a }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct UnaryStatement {
    pub source_a: Register,
}

impl UnaryStatement {
    pub fn new(source_a: Register) -> Self {
        Self { source_a }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct BinaryStatement {
    pub source_a: Register,
    pub source_b: Register,
}

impl BinaryStatement {
    pub fn new(source_a: Register, source_b: Register) -> Self {
        Self { source_a, source_b }
    }
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum LoadSource {
    Constant(u16),
    RAM { address_register: Register },
    Pgm,
}

#[derive(Debug, PartialEq, Eq)]
pub enum JumpTarget {
    Constant(u16),
    Register(Register),
    Label(LabelReference),
}

#[derive(Debug, PartialEq, Eq)]
pub enum JumpCondition {
    True,
    Zero,
    NotZero,
    Less,
    Overflow,
}
