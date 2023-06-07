use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

pub trait LineNumber {
    fn get_line_number(&self) -> u16;
}

#[derive(Debug)]
pub enum Keyword {
    Mmenonic {
        name: String,
        line_number: u16,
    },
    RegisterAddress {
        name: String,
        line_number: u16,
    },
    MemoryAddress {
        address: u16,
        line_number: u16,
        origin: String,
    },
    Constant {
        value: u16,
        line_number: u16,
        origin: String,
    },
    Label {
        name: String,
        line_number: u16,
    },
}

impl Keyword {
    pub fn get_original_string(&self) -> String {
        match &self {
            Keyword::Mmenonic { name, .. } => name.clone(),
            Keyword::RegisterAddress { name, .. } => name.clone(),
            Keyword::Label { name, .. } => name.clone(),
            Keyword::Constant { origin, .. } => origin.clone(),
            Keyword::MemoryAddress { origin, .. } => origin.clone(),
        }
    }
}

impl LineNumber for Keyword {
    fn get_line_number(&self) -> u16 {
        match *self {
            Keyword::Mmenonic { line_number, .. } => line_number,
            Keyword::RegisterAddress { line_number, .. } => line_number,
            Keyword::MemoryAddress { line_number, .. } => line_number,
            Keyword::Constant { line_number, .. } => line_number,
            Keyword::Label { line_number, .. } => line_number,
        }
    }
}

#[derive(Debug)]
pub enum LexerError {
    InvalidRegisterIdentifier {
        actual: String,
        line_number: u16,
    },
    InvalidIdentifier {
        actual: String,
        line_number: u16,
    },
    CommandAfterCommand {
        command_name: String,
        line_number: u16,
    },
    LabelAfterCommand {
        label_name: String,
        line_number: u16,
    },
    CouldNotParseMemoryAddress {
        parsed_word: String,
        line_number: u16,
    },
    IoError(io::Error),
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            LexerError::IoError(io_error) => write!(f, "IO error '{}'", io_error),
            LexerError::InvalidIdentifier {
                actual,
                line_number,
            } => write!(
                f,
                "Invalid identifier '{}' found at line {}",
                actual, line_number
            ),
            LexerError::LabelAfterCommand {
                label_name,
                line_number,
            } => write!(
                f,
                "Found illegal label '{}' after command at line {}",
                label_name, line_number
            ),
            LexerError::CommandAfterCommand {
                command_name,
                line_number,
            } => write!(
                f,
                "Found illegal command '{}' after command at line {}",
                command_name, line_number
            ),
            LexerError::InvalidRegisterIdentifier {
                actual,
                line_number,
            } => write!(
                f,
                "Invalid register identifier '{}' found at line {}",
                actual, line_number
            ),
            LexerError::CouldNotParseMemoryAddress {
                parsed_word,
                line_number,
            } => write!(
                f,
                "Could not parse '{}' to MemoryAddress at line {}",
                parsed_word, line_number
            ),
        }
    }
}

pub fn lexer(path: &Path) -> Result<Vec<Keyword>, Vec<LexerError>> {
    let mut errors: Vec<LexerError> = Vec::new();
    let file: File = File::open(path).map_err(|io_err| vec![LexerError::IoError(io_err)])?;
    let reader = io::BufReader::new(file);
    let mut line_number = 0;
    let mut lexed: Vec<Keyword> = Vec::with_capacity(32);
    let mut keyword_buffer: Vec<Keyword> = Vec::with_capacity(4);

    for line in reader.lines() {
        match line {
            Ok(line) => {
                match lex_line(&mut keyword_buffer, line, line_number) {
                    Ok(_) => lexed.append(&mut keyword_buffer),
                    Err(error) => errors.push(error),
                };
                line_number += 1;
            }
            Err(io_err) => {
                errors.push(LexerError::IoError(io_err));
                return Err(errors);
            }
        }
    }

    lexed.push(Keyword::Mmenonic {
        name: String::from("hlt"),
        line_number,
    });

    Ok(lexed)
}

pub fn lex_line(
    keywords: &mut Vec<Keyword>,
    line: String,
    line_number: u16,
) -> Result<(), LexerError> {
    // starts with 4 spaces -> instruction
    if let Some(line) = line.strip_prefix("    ") {
        let mut args: VecDeque<&str> = line.split(' ').collect();
        let command = args.pop_front().unwrap_or("");
        if command.is_empty() {
            return Ok(());
        }

        keywords.push(Keyword::Mmenonic {
            name: command.to_string(),
            line_number,
        });

        while let Some(word) = args.pop_front() {
            match word_type(word, line_number) {
                Ok(Keyword::Mmenonic { name, line_number }) => {
                    return Err(LexerError::CommandAfterCommand {
                        command_name: name,
                        line_number,
                    })
                }
                Ok(keyword) => keywords.push(keyword),
                Err(err) => return Err(err),
            };
        }
    }
    // starts with . -> label
    if let Some(label) = line.strip_prefix('.') {
        keywords.push(Keyword::Label {
            name: label.to_string(),
            line_number,
        });
    }

    Ok(())
}

fn word_type(word: &str, line_number: u16) -> Result<Keyword, LexerError> {
    // label
    if let Some(label_name) = word.strip_prefix('.') {
        return Ok(Keyword::Label {
            name: String::from(label_name),
            line_number,
        });
    }

    // register address
    if let Some(register_identifier) = word.strip_prefix('%') {
        return Ok(Keyword::RegisterAddress {
            name: String::from(register_identifier),
            line_number,
        });
    }

    // memory address
    if let Some(address_word) = word.strip_prefix('$') {
        if let Ok(address) = u16::from_str_radix(address_word, 16) {
            return Ok(Keyword::MemoryAddress {
                address,
                line_number,
                origin: String::from(address_word),
            });
        } else {
            return Err(LexerError::CouldNotParseMemoryAddress {
                parsed_word: String::from(address_word),
                line_number,
            });
        }
    }

    // constant
    // e.g.: 0xa7, 173, 0b0011010
    if let Some(parsed) = if let Some(signed_hex_word) = word.strip_prefix("-0x") {
        Some((signed_hex_word, 16, true))
    } else if let Some(hex_word) = word.strip_prefix("0x") {
        Some((hex_word, 16, false))
    } else if let Some(signed_binary_word) = word.strip_prefix("-0b") {
        Some((signed_binary_word, 2, true))
    } else if let Some(binary_word) = word.strip_prefix("0b") {
        Some((binary_word, 2, false))
    } else if word
        .chars()
        .next()
        .and_then(|first_char| {
            if first_char.is_ascii_digit() {
                Some(())
            } else {
                None
            }
        })
        .is_some()
    {
        Some((word, 10, false))
    } else if let Some(signed_dec_word) = word.strip_prefix('-') {
        if signed_dec_word
            .chars()
            .next()
            .and_then(|first_char| {
                if first_char.is_ascii_digit() {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
        {
            Some((signed_dec_word, 10, true))
        } else {
            None
        }
    } else {
        None
    }
    .and_then(|(word, radix, sign)| {
        u16::from_str_radix(word, radix)
            .map(|num| if sign { num.wrapping_neg() } else { num })
            .ok()
    }) {
        return Ok(Keyword::Constant {
            value: parsed,
            line_number,
            origin: String::from(word),
        });
    }

    // mmenonic
    if word.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(Keyword::Mmenonic {
            name: String::from(word),
            line_number,
        });
    }

    Err(LexerError::InvalidIdentifier {
        actual: String::from(word),
        line_number,
    })
}
