use std::collections::HashMap;
use std::slice::Iter;

use crate::ir;
use crate::lexer::{Keyword, LineNumber};

pub enum ParserError {
    EndOfStream,
    EmptyStream,
    UnknownCommand {
        command: String,
        line_number: u16,
    },
    MissingArgument {
        command: String,
        arg_name: String,
        line_number: u16,
    },
    CouldNotParseArgument {
        command: String,
        arg_name: String,
        arg_value: String,
        line_number: u16,
    },
    ExpectedFound {
        expected: String,
        found: String,
        line_number: u16,
    },
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParserError::EndOfStream => write!(f, "Reached end of keyword stream"),
            ParserError::EmptyStream => write!(f, "No keywords provided to Parser"),
            ParserError::UnknownCommand {
                command,
                line_number,
            } => write!(f, "Unknown command: '{}' at line {}", command, line_number),

            ParserError::MissingArgument {
                command,
                arg_name,
                line_number,
            } => write!(
                f,
                "Missing argument '{}' in command '{}' at line {}",
                arg_name, command, line_number
            ),
            ParserError::CouldNotParseArgument {
                command,
                arg_name,
                arg_value,
                line_number,
            } => write!(
                f,
                "Invalid value '{}' for argument '{}' in command '{}' at line {}",
                arg_value, arg_name, command, line_number
            ),
            ParserError::ExpectedFound {
                expected,
                found,
                line_number,
            } => write!(
                f,
                "Expected '{}' found '{}' at line {}",
                expected, found, line_number
            ),
        }
    }
}

impl std::fmt::Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for ParserError {}

pub fn parser(keywords: Vec<Keyword>) -> Result<ir::IR, ParserError> {
    let mut known_labels = ir::LabelLUT::with_capacity(10);
    let mut parsed: HashMap<ir::LabelReference, Vec<ir::Instruction>> = HashMap::with_capacity(10);
    let mut iter = keywords.iter();
    let default_label = ir::LabelDefinition::new("main", 0);

    let start_label: ir::LabelDefinition;
    let mut instructions_since_label = 0;

    if let Some(first_keyword) = iter.next() {
        if let Ok(parsed_start_label) = try_parse_label_definition(first_keyword, 0, 0) {
            start_label = parsed_start_label;
        } else {
            start_label = default_label;
            match try_parse_instruction(first_keyword, &mut iter) {
                Ok(instruction) => {
                    if let Some(vec) = parsed.get_mut(&start_label.clone().into()) {
                        vec.push(instruction);
                    } else {
                        parsed.insert(start_label.clone().into(), vec![instruction]);
                    }
                    instructions_since_label += 1;
                }
                Err(ParserError::EndOfStream) => {
                    return Err(ParserError::EmptyStream);
                }
                Err(parser_error) => return Err(parser_error),
            }
        }
    } else {
        return Err(ParserError::EmptyStream);
    }

    known_labels
        .0
        .insert(start_label.clone().into(), start_label.clone());
    let mut last_label: ir::LabelDefinition = start_label.clone();

    loop {
        if let Some(next_keyword) = iter.next() {
            if let Ok(label) = try_parse_label_definition(
                next_keyword,
                last_label.address.0,
                instructions_since_label,
            ) {
                parsed.insert(label.clone().into(), Vec::new());
                known_labels.0.insert(label.clone().into(), label.clone());
                last_label = label;
                instructions_since_label = 0;
            } else {
                match try_parse_instruction(next_keyword, &mut iter) {
                    Ok(instruction) => {
                        if let Some(vec) = parsed.get_mut(&last_label.clone().into()) {
                            vec.push(instruction);
                        } else {
                            parsed.insert(last_label.clone().into(), vec![instruction]);
                        }
                        instructions_since_label += 1;
                    }
                    Err(ParserError::EndOfStream) => {
                        return Ok(ir::IR {
                            start_label: start_label.into(),
                            label_definitions: known_labels,
                            instructions: parsed,
                        })
                    }
                    Err(parser_error) => return Err(parser_error),
                }
            }
        } else {
            return Ok(ir::IR {
                start_label: start_label.into(),
                label_definitions: known_labels,
                instructions: parsed,
            });
        }
    }
}

fn try_parse_instruction(
    next_keyword: &Keyword,
    keywords: &mut Iter<Keyword>,
) -> Result<ir::Instruction, ParserError> {
    match next_keyword {
        Keyword::Mmenonic { name, line_number } => match name.as_str() {
            "ldc" => try_parse_ldc(keywords, *line_number),
            "add" => Ok(ir::Instruction::Add(try_parse_binary_expression(
                "add",
                keywords,
                *line_number,
            )?)),
            "add3" => Ok(ir::Instruction::Add3(try_parse_ternary_expression(
                "add3",
                keywords,
                *line_number,
            )?)),
            "addc" => Ok(ir::Instruction::AddWithCarry(try_parse_binary_expression(
                "addc",
                keywords,
                *line_number,
            )?)),
            "sub" => Ok(ir::Instruction::Subtract(try_parse_binary_expression(
                "sub",
                keywords,
                *line_number,
            )?)),
            "subc" => Ok(ir::Instruction::SubtractWithCarry(
                try_parse_binary_expression("subc", keywords, *line_number)?,
            )),
            "inc" => {
                let unary_statement = try_parse_unary_statement("inc", keywords, *line_number)?;
                Ok(ir::Instruction::Increment(ir::UnaryExpression::new(
                    unary_statement.source_a,
                    unary_statement.source_a,
                )))
            }
            "dec" => {
                let unary_statement = try_parse_unary_statement("dec", keywords, *line_number)?;
                Ok(ir::Instruction::Decrement(ir::UnaryExpression::new(
                    unary_statement.source_a,
                    unary_statement.source_a,
                )))
            }
            "mul" => Ok(ir::Instruction::Multiply(try_parse_binary_expression(
                "mul",
                keywords,
                *line_number,
            )?)),
            "and" => Ok(ir::Instruction::AND(try_parse_binary_expression(
                "and",
                keywords,
                *line_number,
            )?)),
            "or" => Ok(ir::Instruction::OR(try_parse_binary_expression(
                "or",
                keywords,
                *line_number,
            )?)),
            "not" => Ok(ir::Instruction::NOT(try_parse_unary_expression(
                "not",
                keywords,
                *line_number,
            )?)),
            "neg" => Ok(ir::Instruction::Negate(try_parse_unary_expression(
                "neg",
                keywords,
                *line_number,
            )?)),
            "xor" => Ok(ir::Instruction::XOR(try_parse_binary_expression(
                "xor",
                keywords,
                *line_number,
            )?)),
            "xnor" => Ok(ir::Instruction::XNOR(try_parse_binary_expression(
                "xnor",
                keywords,
                *line_number,
            )?)),
            "shl" => Ok(ir::Instruction::ShiftLeft(try_parse_binary_expression(
                "shl",
                keywords,
                *line_number,
            )?)),
            "shr" => Ok(ir::Instruction::ShiftRight(try_parse_binary_expression(
                "shr",
                keywords,
                *line_number,
            )?)),
            "tst" => Ok(ir::Instruction::Test(try_parse_binary_statement(
                "tst",
                keywords,
                *line_number,
            )?)),
            "mov" => Ok(ir::Instruction::Move(try_parse_unary_expression(
                "mov",
                keywords,
                *line_number,
            )?)),
            "s32b" => {
                if let Some(maybe_bool) = keywords.next() {
                    if let Ok(boolean) = try_parse_bool(maybe_bool) {
                        Ok(ir::Instruction::Set32BitMode { enable: boolean })
                    } else {
                        Err(ParserError::CouldNotParseArgument {
                            command: String::from("s32b"),
                            arg_name: String::from("EnableBoolean"),
                            arg_value: maybe_bool.get_original_string(),
                            line_number: *line_number,
                        })
                    }
                } else {
                    Err(ParserError::MissingArgument {
                        command: String::from("s32b"),
                        arg_name: String::from("EnableBoolean"),
                        line_number: *line_number,
                    })
                }
            }
            "hlt" => Ok(ir::Instruction::Halt),
            "jmp" => try_parse_jmp(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::True,
            ),
            "jz" => try_parse_jmp(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Zero,
            ),
            "jnz" => try_parse_jmp(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::NotZero,
            ),
            "jc" => try_parse_jmp(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Less,
            ),
            "jo" => try_parse_jmp(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Overflow,
            ),
            "jrcon" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::True,
            ),
            "jr" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::True,
            ),
            "jzr" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Zero,
            ),
            "jnzr" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::NotZero,
            ),
            "jcr" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Less,
            ),
            "jor" => try_parse_jr(
                next_keyword,
                keywords,
                *line_number,
                ir::JumpCondition::Overflow,
            ),
            "st" => {
                let u_expr = try_parse_unary_expression("st", keywords, *line_number)?;
                Ok(ir::Instruction::StoreRAM {
                    address_register: u_expr.target.address,
                    data_register: u_expr.source_a.address,
                })
            }
            "ld" => {
                let u_expr = try_parse_unary_expression("ld", keywords, *line_number)?;
                Ok(ir::Instruction::Load {
                    address: u_expr.target.address,
                    source: ir::LoadSource::RAM {
                        address_register: u_expr.source_a,
                    },
                })
            }
            "nop" => Ok(ir::Instruction::Noop),
            unknown => Err(ParserError::UnknownCommand {
                command: unknown.to_string(),
                line_number: *line_number,
            }),
        },
        Keyword::Constant {
            value,
            line_number,
            origin: _,
        } => Err(ParserError::UnknownCommand {
            command: format!("{}", value),
            line_number: *line_number,
        }),
        Keyword::Boolean {
            value,
            line_number,
            origin: _,
        } => Err(ParserError::UnknownCommand {
            command: format!("{}", value),
            line_number: *line_number,
        }),
        Keyword::Label { name, line_number } => Err(ParserError::UnknownCommand {
            command: name.to_string(),
            line_number: *line_number,
        }),
        Keyword::RegisterAddress { name, line_number } => Err(ParserError::UnknownCommand {
            command: name.to_string(),
            line_number: *line_number,
        }),
    }
}

/// **ldc** `$TargetRegister` `Constant16`
fn try_parse_ldc(
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::Instruction, ParserError> {
    if let Some(maybe_target_register) = keywords.next() {
        let target_register = try_parse_register(maybe_target_register)?;
        if let Some(maybe_constant) = keywords.next() {
            let constant = try_parse_constant(maybe_constant)?;
            Ok(ir::Instruction::Load {
                address: target_register,
                source: ir::LoadSource::Constant(constant.0),
            })
        } else {
            Err(ParserError::MissingArgument {
                command: String::from("ldc"),
                arg_name: String::from("Constant16"),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: String::from("ldc"),
            arg_name: String::from("TargetRegister"),
            line_number,
        })
    }
}

/// **instruction** `$TargetRegister` `$SourceRegister`
fn try_parse_unary_expression(
    instruction: &str,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::UnaryExpression, ParserError> {
    if let Some(maybe_target_register) = keywords.next() {
        let target = ir::Register::new(try_parse_register(maybe_target_register)?);
        if let Some(maybe_source_register) = keywords.next() {
            let source = ir::Register::new(try_parse_register(maybe_source_register)?);
            Ok(ir::UnaryExpression::new(target, source))
        } else {
            Err(ParserError::MissingArgument {
                command: String::from(instruction),
                arg_name: String::from("SourceRegister"),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: String::from(instruction),
            arg_name: String::from("TargetRegister"),
            line_number,
        })
    }
}

/// **instruction** $SourceRegister`
fn try_parse_unary_statement(
    instruction: &str,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::UnaryStatement, ParserError> {
    if let Some(maybe_source_register) = keywords.next() {
        let source = ir::Register::new(try_parse_register(maybe_source_register)?);
        Ok(ir::UnaryStatement::new(source))
    } else {
        Err(ParserError::MissingArgument {
            command: String::from(instruction),
            arg_name: String::from("SourceRegister"),
            line_number,
        })
    }
}

/// **instruction** `$TargetRegister` `$SourceRegisterA` `$SourceRegisterB`
fn try_parse_binary_expression(
    instruction: &str,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::BinaryExpression, ParserError> {
    if let Some(maybe_target_register) = keywords.next() {
        let target = ir::Register::new(try_parse_register(maybe_target_register)?);
        if let Some(maybe_source_a) = keywords.next() {
            let source_a = ir::Register::new(try_parse_register(maybe_source_a)?);
            if let Some(maybe_source_b) = keywords.next() {
                let source_b = ir::Register::new(try_parse_register(maybe_source_b)?);
                Ok(ir::BinaryExpression::new(target, source_a, source_b))
            } else {
                Err(ParserError::MissingArgument {
                    command: String::from(instruction),
                    arg_name: String::from("SourceRegisterB"),
                    line_number,
                })
            }
        } else {
            Err(ParserError::MissingArgument {
                command: String::from(instruction),
                arg_name: String::from("SourceRegisterA"),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: String::from(instruction),
            arg_name: String::from("TargetRegister"),
            line_number,
        })
    }
}

/// **instruction** $SourceRegisterA` `$SourceRegisterB`
fn try_parse_binary_statement(
    instruction: &str,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::BinaryStatement, ParserError> {
    if let Some(maybe_source_a) = keywords.next() {
        let source_a = ir::Register::new(try_parse_register(maybe_source_a)?);
        if let Some(maybe_source_b) = keywords.next() {
            let source_b = ir::Register::new(try_parse_register(maybe_source_b)?);
            Ok(ir::BinaryStatement::new(source_a, source_b))
        } else {
            Err(ParserError::MissingArgument {
                command: String::from(instruction),
                arg_name: String::from("SourceRegisterB"),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: String::from(instruction),
            arg_name: String::from("SourceRegisterA"),
            line_number,
        })
    }
}

/// **instruction** `$TargetRegister` `$SourceRegisterA` `$SourceRegisterB` `$SourceRegisterC`
fn try_parse_ternary_expression(
    instruction: &str,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::TernaryExpression, ParserError> {
    if let Some(maybe_target_register) = keywords.next() {
        let target = ir::Register::new(try_parse_register(maybe_target_register)?);
        if let Some(maybe_source_a) = keywords.next() {
            let source_a = ir::Register::new(try_parse_register(maybe_source_a)?);
            if let Some(maybe_source_b) = keywords.next() {
                let source_b = ir::Register::new(try_parse_register(maybe_source_b)?);
                if let Some(maybe_source_c) = keywords.next() {
                    let source_c = ir::Register::new(try_parse_register(maybe_source_c)?);
                    Ok(ir::TernaryExpression::new(
                        target, source_a, source_b, source_c,
                    ))
                } else {
                    Err(ParserError::MissingArgument {
                        command: String::from(instruction),
                        arg_name: String::from("SourceRegisterC"),
                        line_number,
                    })
                }
            } else {
                Err(ParserError::MissingArgument {
                    command: String::from(instruction),
                    arg_name: String::from("SourceRegisterB"),
                    line_number,
                })
            }
        } else {
            Err(ParserError::MissingArgument {
                command: String::from(instruction),
                arg_name: String::from("SourceRegisterA"),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: String::from(instruction),
            arg_name: String::from("TargetRegister"),
            line_number,
        })
    }
}

/// **jmp** `%DestinationRegister`
fn try_parse_jmp(
    jump_instruction: &Keyword,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
    condition: ir::JumpCondition,
) -> Result<ir::Instruction, ParserError> {
    if let Some(maybe_target) = keywords.next() {
        if let Ok(register) = try_parse_register(maybe_target) {
            Ok(ir::Instruction::Jump {
                target: ir::JumpTarget::Register(ir::Register::new(register)),
                condition,
            })
        } else {
            Err(ParserError::CouldNotParseArgument {
                command: jump_instruction.get_original_string(),
                arg_name: String::from("DestinationRegister"),
                arg_value: maybe_target.get_original_string(),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: jump_instruction.get_original_string(),
            arg_name: String::from("DestinationRegister"),
            line_number,
        })
    }
}

/// **jr** `ConstantSigned12`
fn try_parse_jr(
    jump_instruction: &Keyword,
    keywords: &mut Iter<Keyword>,
    line_number: u16,
    condition: ir::JumpCondition,
) -> Result<ir::Instruction, ParserError> {
    if let Some(maybe_target) = keywords.next() {
        if let Ok(constant) = try_parse_constant(maybe_target) {
            Ok(ir::Instruction::Jump {
                target: ir::JumpTarget::Constant(constant.0),
                condition,
            })
        } else if let Ok(label) = try_parse_label_reference(maybe_target) {
            Ok(ir::Instruction::Jump {
                target: ir::JumpTarget::Label(label),
                condition,
            })
        } else {
            Err(ParserError::CouldNotParseArgument {
                command: jump_instruction.get_original_string(),
                arg_name: String::from("ConstantSigned12 or JumpLabel"),
                arg_value: maybe_target.get_original_string(),
                line_number,
            })
        }
    } else {
        Err(ParserError::MissingArgument {
            command: jump_instruction.get_original_string(),
            arg_name: String::from("ConstantSigned12 or JumpLabel"),
            line_number,
        })
    }
}

fn try_parse_bool(keyword: &Keyword) -> Result<ir::Boolean, ParserError> {
    match keyword {
        &Keyword::Boolean { value, .. } => Ok(ir::Boolean(value)),
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::Boolean"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

fn try_parse_constant(keyword: &Keyword) -> Result<ir::Constant, ParserError> {
    match keyword {
        &Keyword::Constant { value, .. } => Ok(ir::Constant(value)),
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::Constant"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

fn try_parse_label_definition(
    keyword: &Keyword,
    last_label_address: u16,
    instructions_since_label: u16,
) -> Result<ir::LabelDefinition, ParserError> {
    match keyword {
        Keyword::Label { name, .. } => Ok(ir::LabelDefinition::new(
            name,
            last_label_address + instructions_since_label,
        )),
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::Label"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

fn try_parse_label_reference(keyword: &Keyword) -> Result<ir::LabelReference, ParserError> {
    match &keyword {
        Keyword::Label { name, .. } => Ok(ir::LabelReference::new(name)),
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::Label"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

fn try_parse_register(keyword: &Keyword) -> Result<ir::RegisterAddress, ParserError> {
    match keyword {
        Keyword::RegisterAddress { name, line_number } => {
            if let Some(register_number) = name.strip_prefix("reg") {
                if register_number.is_empty() {
                    None
                } else {
                    let char = register_number.chars().next().unwrap();
                    match register_number.chars().next().unwrap() {
                        '0'..='7' => Some(char.to_digit(8).unwrap()),
                        'A'..='H' => Some(u32::from(char) - u32::from('A')),
                        _ => None,
                    }
                }
                .ok_or(ParserError::ExpectedFound {
                    expected: String::from("valid register number (0..7 | A..H)"),
                    found: register_number.to_string(),
                    line_number: *line_number,
                })
            } else {
                Err(ParserError::ExpectedFound {
                    expected: String::from("valid register identifier"),
                    found: name.to_string(),
                    line_number: *line_number,
                })
            }
            .and_then(|address_u32| {
                u8::try_from(address_u32).or(Err(ParserError::ExpectedFound {
                    expected: String::from("valid register identifier"),
                    found: name.to_string(),
                    line_number: *line_number,
                }))
            })
            .map(ir::RegisterAddress)
        }
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::RegisterAddress"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_instructions() {
        let lexed = vec![
            Keyword::mmenonic("ldc", 0),
            Keyword::register_address("reg0", 0),
            Keyword::constant("0x00", 0, 0),
            Keyword::mmenonic("ldc", 1),
            Keyword::register_address("reg7", 1),
            Keyword::constant("-0x01", 1u16.wrapping_neg(), 1),
            Keyword::mmenonic("ldc", 2),
            Keyword::register_address("regA", 2),
            Keyword::constant("42", 42, 2),
            Keyword::mmenonic("ldc", 3),
            Keyword::register_address("regH", 3),
            Keyword::constant("-1337", 1337u16.wrapping_neg(), 3),
            Keyword::mmenonic("add", 4),
            Keyword::register_address("reg0", 4),
            Keyword::register_address("reg1", 4),
            Keyword::register_address("reg2", 4),
            Keyword::mmenonic("add3", 5),
            Keyword::register_address("reg3", 5),
            Keyword::register_address("reg4", 5),
            Keyword::register_address("reg5", 5),
            Keyword::register_address("reg6", 5),
            Keyword::mmenonic("addc", 6),
            Keyword::register_address("regB", 6),
            Keyword::register_address("regC", 6),
            Keyword::register_address("regD", 6),
            Keyword::mmenonic("sub", 7),
            Keyword::register_address("regE", 7),
            Keyword::register_address("regF", 7),
            Keyword::register_address("regG", 7),
            Keyword::mmenonic("subc", 8),
            Keyword::register_address("reg0", 8),
            Keyword::register_address("reg0", 8),
            Keyword::register_address("reg0", 8),
            Keyword::mmenonic("inc", 9),
            Keyword::register_address("reg0", 9),
            Keyword::mmenonic("dec", 10),
            Keyword::register_address("reg0", 10),
            Keyword::mmenonic("mul", 11),
            Keyword::register_address("reg0", 11),
            Keyword::register_address("reg0", 11),
            Keyword::register_address("reg0", 11),
            Keyword::mmenonic("and", 12),
            Keyword::register_address("reg0", 12),
            Keyword::register_address("reg0", 12),
            Keyword::register_address("reg0", 12),
            Keyword::mmenonic("or", 13),
            Keyword::register_address("reg0", 13),
            Keyword::register_address("reg0", 13),
            Keyword::register_address("reg0", 13),
            Keyword::mmenonic("not", 14),
            Keyword::register_address("reg0", 14),
            Keyword::register_address("reg0", 14),
            Keyword::mmenonic("neg", 15),
            Keyword::register_address("reg0", 15),
            Keyword::register_address("reg0", 15),
            Keyword::mmenonic("xor", 16),
            Keyword::register_address("reg0", 16),
            Keyword::register_address("reg0", 16),
            Keyword::register_address("reg0", 16),
            Keyword::mmenonic("xnor", 17),
            Keyword::register_address("reg0", 17),
            Keyword::register_address("reg0", 17),
            Keyword::register_address("reg0", 17),
            Keyword::mmenonic("shl", 18),
            Keyword::register_address("reg0", 18),
            Keyword::register_address("reg0", 18),
            Keyword::register_address("reg0", 18),
            Keyword::mmenonic("shr", 19),
            Keyword::register_address("reg0", 19),
            Keyword::register_address("reg0", 19),
            Keyword::register_address("reg0", 19),
            Keyword::mmenonic("tst", 20),
            Keyword::register_address("reg0", 20),
            Keyword::register_address("reg0", 20),
            Keyword::mmenonic("mov", 21),
            Keyword::register_address("reg0", 21),
            Keyword::register_address("reg0", 21),
            Keyword::mmenonic("jmp", 22),
            Keyword::register_address("reg0", 22),
            Keyword::mmenonic("jz", 23),
            Keyword::register_address("reg0", 23),
            Keyword::mmenonic("jnz", 24),
            Keyword::register_address("reg0", 24),
            Keyword::mmenonic("jc", 25),
            Keyword::register_address("reg0", 25),
            Keyword::mmenonic("jrcon", 26),
            Keyword::constant("2047", 2047, 26),
            Keyword::mmenonic("jr", 27),
            Keyword::constant("-2047", 2047u16.wrapping_neg(), 27),
            Keyword::label("jump", 28),
            Keyword::mmenonic("jzr", 29),
            Keyword::label("jump", 29),
            Keyword::mmenonic("jnzr", 30),
            Keyword::constant("5", 5, 30),
            Keyword::mmenonic("jcr", 31),
            Keyword::constant("5", 5, 31),
            Keyword::mmenonic("st", 32),
            Keyword::register_address("reg0", 32),
            Keyword::register_address("reg1", 32),
            Keyword::mmenonic("ld", 33),
            Keyword::register_address("reg5", 33),
            Keyword::register_address("reg4", 33),
            Keyword::mmenonic("nop", 34),
            Keyword::mmenonic("hlt", 35),
        ];

        let expected_instructions = vec![
            (
                ir::LabelReference::new("main"),
                vec![
                    ir::Instruction::Load {
                        address: ir::RegisterAddress(0),
                        source: ir::LoadSource::Constant(0x00),
                    },
                    ir::Instruction::Load {
                        address: ir::RegisterAddress(7),
                        source: ir::LoadSource::Constant(0x01u16.wrapping_neg()),
                    },
                    ir::Instruction::Load {
                        address: ir::RegisterAddress(0),
                        source: ir::LoadSource::Constant(42),
                    },
                    ir::Instruction::Load {
                        address: ir::RegisterAddress(7),
                        source: ir::LoadSource::Constant(1337u16.wrapping_neg()),
                    },
                    ir::Instruction::Add(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(1)),
                        source_b: ir::Register::new(ir::RegisterAddress(2)),
                    }),
                    ir::Instruction::Add3(ir::TernaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(3)),
                        source_a: ir::Register::new(ir::RegisterAddress(4)),
                        source_b: ir::Register::new(ir::RegisterAddress(5)),
                        source_c: ir::Register::new(ir::RegisterAddress(6)),
                    }),
                    ir::Instruction::AddWithCarry(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(1)),
                        source_a: ir::Register::new(ir::RegisterAddress(2)),
                        source_b: ir::Register::new(ir::RegisterAddress(3)),
                    }),
                    ir::Instruction::Subtract(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(4)),
                        source_a: ir::Register::new(ir::RegisterAddress(5)),
                        source_b: ir::Register::new(ir::RegisterAddress(6)),
                    }),
                    ir::Instruction::SubtractWithCarry(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Increment(ir::UnaryExpression {
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        target: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Decrement(ir::UnaryExpression {
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        target: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Multiply(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::AND(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::OR(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::NOT(ir::UnaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Negate(ir::UnaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::XOR(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::XNOR(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::ShiftLeft(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::ShiftRight(ir::BinaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Test(ir::BinaryStatement {
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                        source_b: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Move(ir::UnaryExpression {
                        target: ir::Register::new(ir::RegisterAddress(0)),
                        source_a: ir::Register::new(ir::RegisterAddress(0)),
                    }),
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Register(ir::Register::new(ir::RegisterAddress(0))),
                        condition: ir::JumpCondition::True,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Register(ir::Register::new(ir::RegisterAddress(0))),
                        condition: ir::JumpCondition::Zero,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Register(ir::Register::new(ir::RegisterAddress(0))),
                        condition: ir::JumpCondition::NotZero,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Register(ir::Register::new(ir::RegisterAddress(0))),
                        condition: ir::JumpCondition::Less,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Constant(2047),
                        condition: ir::JumpCondition::True,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Constant(2047u16.wrapping_neg()),
                        condition: ir::JumpCondition::True,
                    },
                ],
            ),
            (
                ir::LabelReference::new("jump"),
                vec![
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Label(ir::LabelReference::new("jump")),
                        condition: ir::JumpCondition::Zero,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Constant(5),
                        condition: ir::JumpCondition::NotZero,
                    },
                    ir::Instruction::Jump {
                        target: ir::JumpTarget::Constant(5),
                        condition: ir::JumpCondition::Less,
                    },
                    ir::Instruction::StoreRAM {
                        address_register: ir::RegisterAddress(0),
                        data_register: ir::RegisterAddress(1),
                    },
                    ir::Instruction::Load {
                        address: ir::RegisterAddress(5),
                        source: ir::LoadSource::RAM {
                            address_register: ir::Register::new(ir::RegisterAddress(4)),
                        },
                    },
                    ir::Instruction::Noop,
                    ir::Instruction::Halt,
                ],
            ),
        ];
        let expected_label_definitions = vec![
            (
                ir::LabelReference::new("main"),
                ir::LabelDefinition::new("main", 0),
            ),
            (
                ir::LabelReference::new("jump"),
                ir::LabelDefinition::new("jump", 28),
            ),
        ];

        let expected = ir::IR {
            start_label: ir::LabelReference::new("main"),
            label_definitions: ir::LabelLUT(expected_label_definitions.into_iter().collect()),
            instructions: expected_instructions.into_iter().collect(),
        };

        let found = parser(lexed).unwrap();

        assert_eq!(
            expected.label_definitions.0.len(),
            found.label_definitions.0.len(),
            "found too few label definitions",
        );
        let mut exp_ld_vec = expected
            .label_definitions
            .0
            .iter()
            .collect::<Vec<(&ir::LabelReference, &ir::LabelDefinition)>>();

        let mut found_ld_vec = found
            .label_definitions
            .0
            .iter()
            .collect::<Vec<(&ir::LabelReference, &ir::LabelDefinition)>>();

        found_ld_vec.sort_by(|a, b| a.1.address.cmp(&b.1.address));
        exp_ld_vec.sort_by(|a, b| a.1.address.cmp(&b.1.address));

        for (expected_label_definition, found_label_definition) in
            exp_ld_vec.iter().zip(found_ld_vec.iter())
        {
            assert_eq!(expected_label_definition.0, found_label_definition.0);
            assert_eq!(
                expected_label_definition.1.address, found_label_definition.1.address,
                "label definitions addresses do not match",
            );
        }

        assert_eq!(
            expected.instructions.len(),
            found.instructions.len(),
            "found too few label references in instructions",
        );

        let mut exp_instr_keys = expected.instructions.keys().collect::<Vec<_>>();
        let mut found_instr_keys = found.instructions.keys().collect::<Vec<_>>();

        exp_instr_keys.sort_by(|a, b| a.name().cmp(b.name()));
        found_instr_keys.sort_by(|a, b| a.name().cmp(b.name()));

        for (expected_key_label, found_key_label) in
            exp_instr_keys.iter().zip(found_instr_keys.iter())
        {
            assert_eq!(expected_key_label, found_key_label);
            for (expected_instruction, found_instruction) in expected
                .instructions
                .get(expected_key_label)
                .unwrap()
                .iter()
                .zip(found.instructions.get(found_key_label).unwrap().iter())
            {
                assert_eq!(
                    expected_instruction, found_instruction,
                    "instructions do not match"
                );
            }
        }

        assert_eq!(
            expected.start_label, found.start_label,
            "start label do not match"
        );
    }
}
