use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    process,
};

use risc20_assembler::{generator, lexer, parser};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Error: No input file provided!");
        process::exit(1);
    } else if args.len() < 3 {
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

    let binary = generator::generator(parsed);
    println!("{:?}", binary);
    if let Ok(output) = File::create(args.get(2).unwrap()) {
        let mut writer = BufWriter::new(output);
        writer
            .write_all("v3.0 hex words plain\n".as_bytes())
            .and_then(|_| {
                for instr_line in binary.chunks(8) {
                    let mut line = String::new();
                    for instr_word in instr_line {
                        line = format!("{line} {instr_word}");
                    }
                    writer.write_all(format!("{}\n", line.trim()).as_bytes())?;
                }
                Ok(())
            })
            .and_then(|_| writer.flush())
            .unwrap_or_else(|err| {
                eprintln!("Error: Could not write to file '{err}'");
                process::exit(1);
            })
    } else {
        eprintln!("Error: Could not open file '{}'.", args.get(2).unwrap());
        process::exit(1);
    }
}
