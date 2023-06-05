use std::collections::HashMap;
use std::slice::Iter;

use crate::ir;
use crate::lexer::{Keyword, LineNumber};

pub enum ParserError {
    EndOfStream,
    UnknownComand {
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
            ParserError::UnknownComand {
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
    let mut parsed: HashMap<ir::Label, Vec<ir::Instruction>> = HashMap::with_capacity(10);
    let mut iter = keywords.iter();
    let default_label = ir::Label::new("main", 0);
    let mut last_label: Option<ir::Label> = None;
    let mut start_label: Option<ir::Label> = None;

    loop {
        if let Some(next_keyword) = iter.next() {
            if let Some(label) = try_parse_label(next_keyword) {
                parsed.insert(label.clone(), Vec::new());
                if start_label.is_none() {
                    start_label = Some(label.clone());
                }
                last_label = Some(label);
            } else {
                match try_parse_instruction(next_keyword, &mut iter) {
                    Ok(instruction) => {
                        if let Some(vec) =
                            parsed.get_mut(last_label.as_ref().unwrap_or(&default_label))
                        {
                            vec.push(instruction);
                        } else {
                            parsed.insert(
                                last_label.clone().unwrap_or(default_label.clone()),
                                vec![instruction],
                            );
                        }
                    }
                    Err(ParserError::EndOfStream) => {
                        return Ok(ir::IR {
                            start_label: start_label.unwrap_or(default_label),
                            instructions: parsed,
                        })
                    }
                    Err(parser_error) => return Err(parser_error),
                }
            }
        } else {
            return Ok(ir::IR {
                start_label: start_label.unwrap_or(default_label),
                instructions: parsed,
            });
        }
    }
}

fn try_parse_label(keyword: &Keyword) -> Option<ir::Label> {
    if let Keyword::Label { name, line_number } = keyword {
        return Some(ir::Label::new(name, *line_number));
    }
    None
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
            "jrcon" => try_parse_jrcon(keywords, *line_number),
            unknown => Err(ParserError::UnknownComand {
                command: unknown.to_string(),
                line_number: *line_number,
            }),
        },
        Keyword::Constant { value, line_number } => Err(ParserError::UnknownComand {
            command: format!("{}", value),
            line_number: *line_number,
        }),
        Keyword::MemoryAddress {
            address,
            line_number,
        } => Err(ParserError::UnknownComand {
            command: format!("{}", address),
            line_number: *line_number,
        }),
        Keyword::Label { name, line_number } => Err(ParserError::UnknownComand {
            command: name.to_string(),
            line_number: *line_number,
        }),
        Keyword::RegisterAddress { name, line_number } => Err(ParserError::UnknownComand {
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

/// **jrcon** `ConstantSigned12`
fn try_parse_jrcon(
    keywords: &mut Iter<Keyword>,
    line_number: u16,
) -> Result<ir::Instruction, ParserError> {
    if let Some(maybe_constant) = keywords.next() {
        let constant = try_parse_constant(maybe_constant)?;
        Ok(ir::Instruction::Jump {
            target: ir::JumpTarget::Constant(constant.0),
            condition: ir::JumpCondition::True,
            negate: false,
        })
    } else {
        Err(ParserError::MissingArgument {
            command: String::from("jrcon"),
            arg_name: String::from("ConstantSigned12"),
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

fn try_parse_register(keyword: &Keyword) -> Result<ir::RegisterAddress, ParserError> {
    match keyword {
        Keyword::RegisterAddress { name, line_number } => {
            let address = match name.as_str() {
                "regA" => Ok(0b000),
                "regB" => Ok(0b001),
                "regC" => Ok(0b010),
                "regD" => Ok(0b011),
                "regE" => Ok(0b100),
                "regF" => Ok(0b101),
                "regG" => Ok(0b110),
                "regH" => Ok(0b111),
                unknown => Err(ParserError::ExpectedFound {
                    expected: String::from("valid register identifier"),
                    found: unknown.to_string(),
                    line_number: *line_number,
                }),
            }?;
            Ok(ir::RegisterAddress(address))
        }
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::RegisterAddress"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}

fn try_parse_memory_address(keyword: &Keyword) -> Result<ir::MemoryAddress, ParserError> {
    match keyword {
        &Keyword::MemoryAddress { address, .. } => Ok(ir::MemoryAddress(address)),
        _ => Err(ParserError::ExpectedFound {
            expected: String::from("Keyword::MemoryAddress"),
            found: format!("{:?}", keyword),
            line_number: keyword.get_line_number(),
        }),
    }
}
