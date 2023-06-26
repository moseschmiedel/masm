use std::{
    fs::File,
    io::{BufWriter, Write},
    process,
};

use clap::Parser;

use masm::{generator, lexer, parser};

#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct Cli {
    /// Output file where binary is stored
    #[arg(short, long = "output")]
    output_path: Option<std::path::PathBuf>,
    /// Enable debug output to stdout
    #[arg(short, long = "debug")]
    debug_enable: bool,

    input_path: std::path::PathBuf,
}

fn main() {
    let cli = Cli::parse();
    let input_path = cli.input_path.canonicalize().unwrap_or_else(|err| {
        eprintln!("Error: Could not find input file:");
        eprintln!("{err}");
        process::exit(1);
    });
    let output_path = cli.output_path.unwrap_or("output.hex".into());

    if cli.debug_enable {
        println!("Input: {}", input_path.display());
        println!("Output: {}", output_path.display());
    }

    let lexed = lexer::lexer(&input_path).unwrap_or_else(|errors| {
        for err in errors {
            eprintln!("Lexer: {err}");
        }
        process::exit(1);
    });

    let parsed = parser::parser(lexed).unwrap_or_else(|err| {
        eprintln!("Parser: {err}");
        process::exit(1);
    });

    if cli.debug_enable {
        println!("{:#?}", parsed.instructions.keys());
        println!("{:#?}", parsed.instructions.values());
    }

    let binary = generator::generator(parsed).unwrap_or_else(|err| {
        eprintln!("Generator: {err}");
        process::exit(1);
    });

    if cli.debug_enable {
        println!("{:#?}", binary);
    }
    let output = File::create(&output_path).unwrap_or_else(|err| {
        eprintln!("Error: Could not open output file for writing:");
        eprintln!("{err}");
        process::exit(1);
    });
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
            eprintln!("Error: Could not write to file:");
            eprintln!("{err}");
            process::exit(1);
        });
}
