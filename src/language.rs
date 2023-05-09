pub enum Command {
    EmptyLine,
    Label {
        label: String,
        address: MemoryAddress,
    },
    Move(UnaryExpression),
    Load {
        source: LoadSource,
    },
    StoreRAM,
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

pub struct RegisterAddress(u8);
pub struct MemoryAddress(pub u16);

pub struct Register {
    adress: RegisterAddress,
}

pub struct UnaryExpression {
    target: Register,
    source_a: Register,
}

pub struct BinaryExpression {
    target: Register,
    source_a: Register,
    source_b: Register,
}

pub struct BinaryStatement {
    source_a: Register,
    source_b: Register,
}

struct TernaryExpression {
    target: Register,
    source_a: Register,
    source_b: Register,
    source_c: Register,
}

pub enum LoadSource {
    Constant(u16),
    RAM { address: MemoryAddress },
    Pgm,
}

pub enum JumpCondition {
    True,
    Zero,
    Less,
}
