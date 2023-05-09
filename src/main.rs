use risc20_assembler::parser::{Parser, ParserError};

fn main() -> Result<(), ParserError> {
    let parser = Parser::new("./test.s")?;
    parser.parse()?;

    Ok(())
}
