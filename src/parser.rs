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
            "inc" => Ok(ir::Instruction::Increment(try_parse_unary_statement(
                "inc",
                keywords,
                *line_number,
            )?)),
            "dec" => Ok(ir::Instruction::Decrement(try_parse_unary_statement(
                "dec",
                keywords,
                *line_number,
            )?)),
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
