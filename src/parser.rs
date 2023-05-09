use std::io;
use std::io::prelude::*;
use std::{collections::VecDeque, fs::File};

use crate::language::{Command, LoadSource, MemoryAddress};

pub struct Parser {
    file: File,
}

pub enum ParserError {
    UnknownComand {
        command: String,
        line_number: u16,
    },
    IoError(io::Error),
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
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            ParserError::UnknownComand {
                command,
                line_number,
            } => {
                write!(
                    f,
                    "Could not parse string: '{}' at line {}",
                    command, line_number
                )?;
                Ok(())
            }
            ParserError::IoError(io_error) => {
                write!(f, "IO Error while parsing: {}", io_error)?;
                Ok(())
            }
            ParserError::MissingArgument {
                command,
                arg_name,
                line_number,
            } => {
                write!(
                    f,
                    "Missing argument '{}' in command '{}' at line {}",
                    arg_name, command, line_number
                )?;
                Ok(())
            }
            ParserError::CouldNotParseArgument {
                command,
                arg_name,
                arg_value,
                line_number,
            } => {
                write!(
                    f,
                    "Invalid value '{}' for argument '{}' in command '{}' at line {}",
                    arg_value, arg_name, command, line_number
                )?;
                Ok(())
            }
        }
    }
}

impl std::fmt::Debug for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<io::Error> for ParserError {
    fn from(io_error: io::Error) -> ParserError {
        ParserError::IoError(io_error)
    }
}

impl std::error::Error for ParserError {}

impl Parser {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(Parser { file })
    }
    pub fn parse(&self) -> Result<Vec<Command>, ParserError> {
        let reader = io::BufReader::new(&self.file);
        let mut line_number = 0;
        let mut parsed: Vec<Command> = Vec::with_capacity(16);

        for line in reader.lines() {
            parsed.push(parse_line(line?, line_number)?);
            line_number += 1;
        }

        Ok(parsed)
    }
}

fn parse_line(line: String, line_number: u16) -> Result<Command, ParserError> {
    // starts with 4 spaces -> normal command

    if let Some(command_line) = line.strip_prefix("    ") {
        let mut args: VecDeque<&str> = command_line.split(" ").collect();
        let command = args.pop_front().unwrap_or("");
        return match command {
            "" => Ok(Command::EmptyLine),
            "ldc" => {
                let constant = args.pop_front().ok_or(ParserError::MissingArgument {
                    command: String::from("ldc"),
                    arg_name: String::from("constant"),
                    line_number,
                })?;
                Ok(Command::Load {
                    source: LoadSource::Constant(constant.parse::<u16>().map_err(|_| {
                        ParserError::CouldNotParseArgument {
                            command: String::from("ldc"),
                            arg_name: String::from("constant"),
                            arg_value: String::from(constant),
                            line_number,
                        }
                    })?),
                })
            }
            _ => Err(ParserError::UnknownComand {
                command: String::from(command),
                line_number,
            }),
        };
    }
    // starts with . -> label
    if let Some(label) = line.strip_prefix(".") {
        return Ok(Command::Label {
            label: String::from(label),
            address: MemoryAddress(line_number),
        });
    }

    Err(ParserError::UnknownComand {
        command: line,
        line_number,
    })
}
