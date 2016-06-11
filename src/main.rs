extern crate pschip8;

use std::path::Path;
use pschip8::Interpreter;

fn main() {
    // 0110 0000 0000 0000 (24576): V0 = 0
    // 0111 0000 0000 0001 (28673): V0 = V0 + 1
    // 0001 0010 0000 0020 (1202): jmp 0x202
    let test_instructions: [u16; 3] = [24576, 28673, 4610];
    let mut intp = Interpreter::init();
    let path = Path::new("/home/francis/Desktop/random_number_test_matthew_mikolay_2010.ch8");
    intp.load_from_bytes(&test_instructions);
    intp.load_program_from_file(&path);
    intp.print_memory();
    intp.run();
}
