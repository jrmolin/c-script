use c_script::codegen::CodeGen;
use c_script::parser::parse_program;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <filename> [-o <output_file>]", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let mut output_file = None;

    let mut i = 2;
    while i < args.len() {
        if args[i] == "-o" {
            if i + 1 < args.len() {
                output_file = Some(&args[i + 1]);
                i += 1;
            } else {
                eprintln!("Error: -o requires an argument");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    match parse_program(&contents) {
        Ok((remaining, program)) => {
            if !remaining.trim().is_empty() {
                eprintln!("Warning: Unparsed input remaining:\n{}", remaining);
            }

            let mut codegen = CodeGen::new();
            let ir = codegen.generate(&program);

            if let Some(out_path) = output_file {
                fs::write(out_path, ir).expect("Unable to write output file");
            } else {
                println!("{}", ir);
            }
        }
        Err(e) => {
            eprintln!("Error parsing file: {:?}", e);
            std::process::exit(1);
        }
    }
}
