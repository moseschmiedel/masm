use std::{collections::VecDeque, fs::File::, io::{self, BufReader}};

pub enum Keyword {
    Mmenonic { name: String, line_number: u16 },
    RegisterAddress { name: String },
    MemoryAddress(u16),
    Constant(u16),
    Label { name: String, line_number: u16 },
}

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
}

pub fn lexer(path: String) -> Result<Vec<Keyword>, Vec<LexerError>> {
    let file = File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut line_number = 0;
    let mut lexed: Vec<Keyword> = Vec::with_capacity(32);
    let mut errors: Vec<LexerError> = Vec::new();

    for line in reader.lines() {
        match lex_line(line?, line_number) {
            Ok(keywords) => lexed.append(&mut keywords),
            Err(error) => errors.push(error),
        };
        line_number += 1;
    }

    Ok(parsed)
}

pub fn lex_line(line: String, line_number: u16) -> Result<Vec<Keyword>, LexerError> {
    let mut keywords: Vec<Keyword> = Vec::with_capacity(4);

    // starts with 4 spaces -> normal command
    if let Some(line) = line.strip_prefix("    ") {
        let mut args: VecDeque<&str> = line.split(" ").collect();
        let command = args.pop_front().unwrap_or("");
        if command == "" {
            return Ok(Vec::new());
        }

        keywords.push(Keyword::Mmenonic {
            name: command.to_string(),
            line_number,
        });

        while let Some(word) = args.pop_front() {
            match word_type(word, line_number) {
                Ok(Keyword::Label { name, line_number }) => {
                    return Err(LexerError::LabelAfterCommand {
                        label_name: name,
                        line_number,
                    })
                }
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
    if let Some(label) = line.strip_prefix(".") {
        keywords.push(Keyword::Label {
            name: label.to_string(),
            line_number,
        });
    }

    Ok(keywords)
}

fn word_type(word: &str, line_number: u16) -> Result<Keyword, LexerError> {
    // label
    if let Some(label_name) = word.strip_prefix(".") {
        return Ok(Keyword::Label {
            name: String::from(label_name),
            line_number,
        });
    }

    // register address
    if let Some(register_identifier) = word.strip_prefix("%") {
        return Ok(Keyword::RegisterAddress {
            name: String::from(register_identifier),
        });
    }

    // memory address
    if let Some(address_word) = word.strip_prefix("$") {
        if let Some(address) = u16::from_str_radix(address_word, 16).ok() {
            return Ok(Keyword::MemoryAddress(address));
        } else {
            return Err(LexerError::CouldNotParseMemoryAddress {
                parsed_word: String::from(address_word),
                line_number,
            });
        }
    }

    // constant
    // e.g.: 0xa7, 173, 0b0011010
    if let Some(parsed) = if let Some(hex_word) = word.strip_prefix("0x") {
        Some((hex_word, 16))
    } else if let Some(binary_word) = word.strip_prefix("0b") {
        Some((binary_word, 2))
    } else if word
        .chars()
        .next()
        .and_then(|first_char| {
            if first_char.is_digit(10) {
                Some(())
            } else {
                None
            }
        })
        .is_some()
    {
        Some((word, 10))
    } else {
        None
    }
    .and_then(|(word, radix)| u16::from_str_radix(word, radix).ok())
    {
        return Ok(Keyword::Constant(parsed));
    }

    // mmenonic
    if word.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Ok(Keyword::Mmenonic {
            name: String::from(word),
            line_number,
        });
    }

    return Err(LexerError::InvalidIdentifier {
        actual: String::from(word),
        line_number,
    });
}
