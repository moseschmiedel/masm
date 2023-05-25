use std::fmt;

use crate::ir::{self, LoadSource};

#[derive(Debug, Clone)]
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
        let upper_12_bit = (constant - lower_4_bit) >> 4;

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
}

impl fmt::Display for InstructionWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for nibble in self.buffer.chunks(4).rev() {
            write!(f, "{}", nibble_to_hex(nibble))?;
        }
        Ok(())
    }
}

const HEX_MAP: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f",
];

fn nibble_to_hex(buffer: &[bool]) -> String {
    let mut byte = 0usize;
    let mut counter = 0;
    for bit in buffer {
        if *bit {
            byte += 2usize.pow(counter);
        }
        counter += 1;
    }
    return HEX_MAP[byte].to_string();
}

fn set_bits(buffer: &mut [bool], int: u32) {
    let mut int = int;

    for bit in buffer {
        *bit = int % 2 == 1;
        int = (int - int % 2) >> 1;
    }
}

pub fn generator(ir: ir::IR) -> Vec<InstructionWord> {
    let mut labels: Vec<&ir::Label> = ir.instructions.keys().collect();
    labels.sort_by(|&a, &b| a.address.cmp(&b.address));

    let mut binary: Vec<InstructionWord> = Vec::with_capacity(32);
    let mut instruction_word = InstructionWord::new();

    for label in labels {
        if let Some(instructions) = ir.instructions.get(label) {
            for instr in instructions {
                instruction_word.clear();
                match instr {
                    &ir::Instruction::Load {
                        address,
                        source: LoadSource::Constant(c),
                    } => {
                        instruction_word.set_load();
                        instruction_word.set_load_address(address.0);
                        instruction_word.set_constant16(c);
                        binary.push(instruction_word.clone());
                    }
                    &ir::Instruction::Add(ir::BinaryExpression {
                        target,
                        source_a,
                        source_b,
                    }) => {
                        instruction_word.set_opcode(0x0);
                        instruction_word.set_target(target.addr());
                        instruction_word.set_op_a(source_a.addr());
                        instruction_word.set_op_b(source_b.addr());
                        binary.push(instruction_word.clone());
                    }
                    &ir::Instruction::Halt => {
                        instruction_word.set_opcode(0x7f);
                        binary.push(instruction_word.clone());
                    }
                    &ir::Instruction::Jump {
                        target: ir::JumpTarget::Constant(c),
                        condition: ir::JumpCondition::True,
                        negate: false,
                    } => {
                        instruction_word.set_opcode(0x80);
                        instruction_word.set_constant12(c);
                        binary.push(instruction_word.clone());
                    }
                    _ => (),
                }
            }
        }
    }

    return binary;
}
