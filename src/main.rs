use std::{env, process};

use risc20_assembler::{lexer, parser};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file provided!");
        process::exit(1);
    }

    let lexed = lexer::lexer(args.get(1).unwrap()).unwrap_or_else(|errors| {
        for err in errors {
            eprintln!("Lexer: {err}");
        }
        process::exit(1);
    });

    let parsed = parser::parser(lexed).unwrap_or_else(|err| {
        eprintln!("Parser: {err}");
        process::exit(1);
    });
    println!("{:?}", parsed.instructions.keys());
    println!("{:?}", parsed.instructions.values());
}
