use std::{
    collections::VecDeque,
    fs::File,
    io::{self, BufRead},
    path::Path,
};

pub trait LineNumber {
    fn get_line_number(&self) -> u16;
}

/// Keywords are the Tokens, that the lexer creates from the
/// input character stream
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
    pub fn mmenonic(name: &str, line_number: u16) -> Keyword {
        Keyword::Mmenonic {
            name: name.to_string(),
            line_number,
        }
    }
    pub fn register_address(name: &str, line_number: u16) -> Keyword {
        Keyword::RegisterAddress {
            name: name.to_string(),
            line_number,
        }
    }
    pub fn constant(origin: &str, value: u16, line_number: u16) -> Keyword {
        Keyword::Constant {
            origin: origin.to_string(),
            value,
            line_number,
        }
    }
    pub fn label(name: &str, line_number: u16) -> Keyword {
        Keyword::Label {
            name: name.to_string(),
            line_number,
        }
    }
    pub fn get_original_string(&self) -> String {
        match &self {
            Keyword::Mmenonic { name, .. } => name.clone(),
            Keyword::RegisterAddress { name, .. } => format!("%{}", name.clone()),
            Keyword::Label { name, .. } => format!(".{}", name.clone()),
            Keyword::Constant { origin, .. } => origin.clone(),
        }
    }
}

impl PartialEq for Keyword {
    fn eq(&self, other: &Keyword) -> bool {
        match (self, other) {
            (
                Keyword::Label {
                    name: name_self, ..
                },
                Keyword::Label {
                    name: name_other, ..
                },
            ) => name_self == name_other,
            (
                Keyword::RegisterAddress {
                    name: name_self, ..
                },
                Keyword::RegisterAddress {
                    name: name_other, ..
                },
            ) => name_self == name_other,
            (
                Keyword::Mmenonic {
                    name: name_self, ..
                },
                Keyword::Mmenonic {
                    name: name_other, ..
                },
            ) => name_self == name_other,
            (
                Keyword::Constant {
                    value: value_self,
                    origin: origin_self,
                    ..
                },
                Keyword::Constant {
                    value: value_other,
                    origin: origin_other,
                    ..
                },
            ) => value_self == value_other && origin_self == origin_other,
            _ => false,
        }
    }
}

impl LineNumber for Keyword {
    fn get_line_number(&self) -> u16 {
        match *self {
            Keyword::Mmenonic { line_number, .. } => line_number,
            Keyword::RegisterAddress { line_number, .. } => line_number,
            Keyword::Constant { line_number, .. } => line_number,
            Keyword::Label { line_number, .. } => line_number,
        }
    }
}

///
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
        }
    }
}

/// The lexer reads the provided assembler text file and separate
/// it into Tokens (Keywords).
/// Tokens are strings that are separated by whitespace.
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

    let hlt = Keyword::Mmenonic {
        name: String::from("hlt"),
        line_number,
    };
    if lexed
        .last()
        .and_then(|last_keyword| if *last_keyword != hlt { Some(()) } else { None })
        .is_some()
    {
        lexed.push(hlt);
    }

    Ok(lexed)
}

pub fn lex_line(
    keywords: &mut Vec<Keyword>,
    line: String,
    line_number: u16,
) -> Result<(), LexerError> {
    let mut line = line;
    // starts with 4 spaces -> instruction
    line = line.trim_end().to_string();
    if line.starts_with([' ', '\t']) {
        line = line.trim_start().to_string();
        if let Some(semi_idx) = line.find(';') {
            line.truncate(semi_idx);
        }
        let mut args: VecDeque<&str> = line.split_whitespace().collect();
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
    // ends with : -> label
    if let Some(label) = line.strip_suffix(':') {
        keywords.push(Keyword::Label {
            name: label.to_string(),
            line_number,
        });
    }

    Ok(())
}

fn word_type(word: &str, line_number: u16) -> Result<Keyword, LexerError> {
    // register address
    if let Some(register_identifier) = word.strip_prefix('%') {
        return Ok(Keyword::RegisterAddress {
            name: String::from(register_identifier),
            line_number,
        });
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
        return Ok(Keyword::Label {
            name: String::from(word),
            line_number,
        });
    }

    Err(LexerError::InvalidIdentifier {
        actual: String::from(word),
        line_number,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_all_instructions() {
        let expected = vec![
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

        let found = lexer(Path::new("tests/all_instructions.s")).unwrap();

        for (expected_keyword, found_keyword) in expected.iter().zip(found.iter()) {
            assert_eq!(expected_keyword, found_keyword);
        }
    }

    #[test]
    fn whitespace() {
        let expected = vec![
            Keyword::mmenonic("ldc", 0),
            Keyword::register_address("reg0", 0),
            Keyword::constant("0x4", 4, 0),
            Keyword::label("loop", 1),
            Keyword::mmenonic("hlt", 2),
        ];

        let found = lexer(Path::new("tests/whitespace.s")).unwrap();
        for (expected_keyword, found_keyword) in expected.iter().zip(found.iter()) {
            assert_eq!(expected_keyword, found_keyword);
        }
    }
}
