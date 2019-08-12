use codegen::{Compiler, Language};

fn main() {
    let mut args = ::std::env::args();
    if args.len() != 3 {
        let prog = args.nth(0).unwrap();
        println!("** Only support Rust, now. **");
        println!("Usage:\n\t{} input.mol output.rs", prog);
    } else {
        let input = args.nth(1).unwrap();
        let output = args.nth(0).unwrap();
        println!("Compile {} to {} ...", input, output);
        let mut compiler = Compiler::new();
        compiler
            .language(Language::Rust)
            .file_path(input)
            .out_dir(output)
            .run();
    }
}
