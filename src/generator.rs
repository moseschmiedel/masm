use std::fmt;

use crate::ir;

#[derive(Clone)]
pub struct InstructionWord {
    buffer: [bool; 20],
}

impl InstructionWord {
    fn new() -> Self {
        Self {
            buffer: [false; 20],
        }
    }
    fn clear(&mut self) {
        self.buffer.fill(false);
    }

    fn set_constant16(&mut self, constant: u16) {
        let lower_4_bit = constant % 16;
        let upper_12_bit = constant >> 4;

        set_bits(&mut self.buffer[0..=3], lower_4_bit as u32);
        set_bits(&mut self.buffer[8..=19], upper_12_bit as u32);
    }
    fn set_load(&mut self) {
        self.buffer[7] = true;
    }
    fn set_load_address(&mut self, address: u8) {
        set_bits(&mut self.buffer[4..=6], address as u32);
    }
    fn set_target(&mut self, address: u8) {
        set_bits(&mut self.buffer[17..=19], address as u32);
    }
    fn set_op_a(&mut self, address: u8) {
        set_bits(&mut self.buffer[8..=10], address as u32);
    }
    fn set_op_b(&mut self, address: u8) {
        set_bits(&mut self.buffer[11..=13], address as u32);
    }
    fn set_op_c(&mut self, address: u8) {
        set_bits(&mut self.buffer[14..=16], address as u32);
    }
    fn set_opcode(&mut self, opcode: u8) {
        set_bits(&mut self.buffer[0..=7], opcode as u32);
    }
    fn set_constant12(&mut self, constant: u16) {
        set_bits(&mut self.buffer[8..=19], constant as u32);
    }
    fn set_unary_statement(&mut self, u_stat: &ir::UnaryStatement) {
        self.set_op_a(u_stat.source_a.addr());
    }
    fn set_unary_expression(&mut self, u_expr: &ir::UnaryExpression) {
        self.set_target(u_expr.target.addr());
        self.set_op_a(u_expr.source_a.addr());
    }
    fn set_binary_statement(&mut self, b_stat: &ir::BinaryStatement) {
        self.set_op_a(b_stat.source_a.addr());
        self.set_op_b(b_stat.source_b.addr());
    }
    fn set_binary_expression(&mut self, b_expr: &ir::BinaryExpression) {
        self.set_target(b_expr.target.addr());
        self.set_op_a(b_expr.source_a.addr());
        self.set_op_b(b_expr.source_b.addr());
    }
    fn set_ternary_expression(&mut self, t_expr: &ir::TernaryExpression) {
        self.set_target(t_expr.target.addr());
        self.set_op_a(t_expr.source_a.addr());
        self.set_op_b(t_expr.source_b.addr());
        self.set_op_c(t_expr.source_c.addr());
    }
}

impl fmt::Display for InstructionWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for nibble in self.buffer.chunks(4).rev() {
            write!(f, "{}", nibble_to_hex(nibble))?;
        }
        Ok(())
    }
}

impl fmt::Debug for InstructionWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "InstructionWord {{ buffer: 0x")?;
        for nibble in self.buffer.chunks(4).rev() {
            write!(f, "{}", nibble_to_hex(nibble))?;
        }
        write!(f, " }}")?;
        Ok(())
    }
}

const HEX_MAP: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];

fn nibble_to_hex(buffer: &[bool]) -> String {
    let mut byte = 0usize;
    for (idx, bit) in buffer.iter().enumerate() {
        if *bit {
            byte += 2usize.pow(idx as u32);
        }
    }
    HEX_MAP[byte].to_string()
}

fn set_bits(buffer: &mut [bool], int: u32) {
    let mut int = int;

    for bit in buffer {
        *bit = int % 2 == 1;
        int >>= 1;
    }
}

pub enum GeneratorError {
    UndefinedLabel { label_name: String },
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneratorError::UndefinedLabel { label_name } => {
                write!(f, "Could not find definition of label '{}'", label_name,)
            }
        }
    }
}

pub fn generator(ir: ir::IR) -> Result<Vec<InstructionWord>, GeneratorError> {
    let mut labels: Vec<&ir::LabelDefinition> = ir.label_definitions.0.values().collect();
    labels.sort_by(|&a, &b| a.address.cmp(&b.address));

    let mut binary: Vec<InstructionWord> = Vec::with_capacity(32);
    let mut instruction_word = InstructionWord::new();

    for label in labels {
        if let Some(instructions) = ir.instructions.get(&label.clone().into()) {
            for (idx, instr) in instructions.iter().enumerate() {
                instruction_word.clear();
                match instr {
                    ir::Instruction::Add(binary_expression) => {
                        instruction_word.set_opcode(0x0);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Add3(ternary_expression) => {
                        instruction_word.set_opcode(0x1);
                        instruction_word.set_ternary_expression(ternary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::AddWithCarry(binary_expression) => {
                        instruction_word.set_opcode(0x2);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Subtract(binary_expression) => {
                        instruction_word.set_opcode(0x3);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::SubtractWithCarry(binary_expression) => {
                        instruction_word.set_opcode(0x4);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Increment(unary_expression) => {
                        instruction_word.set_opcode(0x5);
                        instruction_word.set_unary_expression(unary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Decrement(unary_expression) => {
                        instruction_word.set_opcode(0x6);
                        instruction_word.set_unary_expression(unary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Multiply(binary_expression) => {
                        instruction_word.set_opcode(0x7);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Test(binary_statement) => {
                        instruction_word.set_opcode(0x8);
                        instruction_word.set_binary_statement(binary_statement);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::AND(binary_expression) => {
                        instruction_word.set_opcode(0x9);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::OR(binary_expression) => {
                        instruction_word.set_opcode(0xa);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::NOT(unary_expression) => {
                        instruction_word.set_opcode(0xb);
                        instruction_word.set_unary_expression(unary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Negate(unary_expression) => {
                        instruction_word.set_opcode(0xb);
                        instruction_word.set_unary_expression(unary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::XOR(binary_expression) => {
                        instruction_word.set_opcode(0xd);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::XNOR(binary_expression) => {
                        instruction_word.set_opcode(0xe);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::ShiftLeft(binary_expression) => {
                        instruction_word.set_opcode(0xf);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::ShiftRight(binary_expression) => {
                        instruction_word.set_opcode(0x10);
                        instruction_word.set_binary_expression(binary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Move(unary_expression) => {
                        instruction_word.set_opcode(0x48);
                        instruction_word.set_unary_expression(unary_expression);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Set32BitMode { enable } => {
                        instruction_word.set_opcode(0x4a);
                        match enable {
                            ir::Boolean(true) => instruction_word.set_constant12(0xff),
                            ir::Boolean(false) => instruction_word.set_constant12(0x00),
                        };
                        binary.push(instruction_word.clone());
                    }
                    // Absolute jumps
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Register(reg),
                        condition,
                    } => {
                        let opcode = 0x50
                            + match condition {
                                ir::JumpCondition::True => 0,
                                ir::JumpCondition::Zero => 1,
                                ir::JumpCondition::NotZero => 2,
                                ir::JumpCondition::Less => 3,
                                ir::JumpCondition::Overflow => 4,
                            };
                        instruction_word.set_opcode(opcode);
                        instruction_word.set_op_a(reg.addr());
                        binary.push(instruction_word.clone());
                    }
                    // Relative Jumps
                    ir::Instruction::Jump { target, condition } => {
                        let opcode = 0x58
                            + match condition {
                                ir::JumpCondition::True => 0,
                                ir::JumpCondition::Zero => 1,
                                ir::JumpCondition::NotZero => 2,
                                ir::JumpCondition::Less => 3,
                                ir::JumpCondition::Overflow => 4,
                            };
                        instruction_word.set_opcode(opcode);
                        let offset = match target {
                            ir::JumpTarget::Label(jump_label_ref) => {
                                if let Some(jump_label) = ir.label_definitions.0.get(jump_label_ref)
                                {
                                    jump_label
                                        .address
                                        .0
                                        .wrapping_sub(label.address.0 + (idx as u16) + 1)
                                } else {
                                    return Err(GeneratorError::UndefinedLabel {
                                        label_name: jump_label_ref.name().to_string(),
                                    });
                                }
                            }
                            ir::JumpTarget::Constant(c) => *c - 1,
                            _ => 0,
                        };
                        instruction_word.set_constant12(offset);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Debug => {
                        instruction_word.set_opcode(0x7e);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Halt => {
                        instruction_word.set_opcode(0x7f);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Load {
                        address,
                        source: ir::LoadSource::Constant(c),
                    } => {
                        instruction_word.set_load();
                        instruction_word.set_load_address(address.0);
                        instruction_word.set_constant16(*c);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::StoreRAM {
                        address_register,
                        data_register,
                    } => {
                        instruction_word.set_opcode(0x68);
                        instruction_word.set_op_a(data_register.0);
                        instruction_word.set_op_b(address_register.0);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Load {
                        address,
                        source: ir::LoadSource::RAM { address_register },
                    } => {
                        instruction_word.set_opcode(0x69);
                        instruction_word.set_op_b(address_register.addr());
                        instruction_word.set_target(address.0);
                        binary.push(instruction_word.clone());
                    }
                    ir::Instruction::Noop => {
                        instruction_word.set_opcode(0x6c);
                        binary.push(instruction_word.clone());
                    }
                    _ => (),
                }
            }
        }
    }

    Ok(binary)
}
