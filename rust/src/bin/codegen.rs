use std::env;
use std::fs;
use c_script::parser::parse_program;
use c_script::codegen::CodeGen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let contents = fs::read_to_string(filename).expect("Something went wrong reading the file");

    match parse_program(&contents) {
        Ok((remaining, program)) => {
            if !remaining.trim().is_empty() {
                eprintln!("Warning: Unparsed input remaining:\n{}", remaining);
            }
            
            let mut codegen = CodeGen::new();
            let ir = codegen.generate(&program);
            println!("{}", ir);
        }
        Err(e) => {
            eprintln!("Error parsing file: {:?}", e);
            std::process::exit(1);
        }
    }
}
