#[macro_use]
extern crate clap;
extern crate pschip8;

use std::path::Path;
use clap::{Arg, App};
use pschip8::Interpreter;

fn main() {
    let matches = App::new("pschip8")
        .version(crate_version!())
        .author("Francis A. <francisagyapong2@gmail.com>")
        .about("Pretty Simple Chip8 Interpreter")
        .arg(Arg::with_name("program")
             .short("p")
             .long("program")
             .value_name("FILE")
             .help("The chip-8 program file")
             .required(true))
        .get_matches();

    let program_path = Path::new(matches.value_of("program").unwrap());
    let mut intp = Interpreter::new();
    intp.load_program_from_file(&program_path);
    intp.run();
}
