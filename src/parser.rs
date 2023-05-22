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
    return None;
}

fn try_parse_instruction(
    next_keyword: &Keyword,
    keywords: &mut Iter<Keyword>,
) -> Result<ir::Instruction, ParserError> {
    match next_keyword {
        Keyword::Mmenonic { name, line_number } => match name.as_str() {
            "ldc" => return try_parse_ldc(keywords, *line_number),
            unknown => {
                return Err(ParserError::UnknownComand {
                    command: unknown.to_string(),
                    line_number: *line_number,
                });
            }
        },
        Keyword::Constant { value, line_number } => {
            return Err(ParserError::UnknownComand {
                command: format!("{}", value),
                line_number: *line_number,
            })
        }
        Keyword::MemoryAddress {
            address,
            line_number,
        } => {
            return Err(ParserError::UnknownComand {
                command: format!("{}", address),
                line_number: *line_number,
            })
        }
        Keyword::Label { name, line_number } => {
            return Err(ParserError::UnknownComand {
                command: name.to_string(),
                line_number: *line_number,
            })
        }
        Keyword::RegisterAddress { name, line_number } => {
            return Err(ParserError::UnknownComand {
                command: name.to_string(),
                line_number: *line_number,
            });
        }
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
            return Ok(ir::Instruction::Load {
                address: target_register,
                source: ir::LoadSource::Constant(constant.0),
            });
        } else {
            return Err(ParserError::MissingArgument {
                command: String::from("ldc"),
                arg_name: String::from("Constant16"),
                line_number,
            });
        }
    } else {
        return Err(ParserError::MissingArgument {
            command: String::from("ldc"),
            arg_name: String::from("TargetRegister"),
            line_number,
        });
    }
}

fn try_parse_constant(keyword: &Keyword) -> Result<ir::Constant, ParserError> {
    match keyword {
        &Keyword::Constant { value, .. } => {
            return Ok(ir::Constant(value));
        }
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
            return Ok(ir::RegisterAddress(address));
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
