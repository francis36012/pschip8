extern crate pschip8;

use pschip8::Interpreter;

fn main() {
    sound_test();
}

fn sound_test() {
    // 0110 0000 0000 1010 (24586): LD V0, 10
    // 1111 0000 0001 1000 (61464): LD ST, V0
    // 0001 0010 0000 0100 (4610):  jmp 0x202
    let test_instructions: [u16; 3] = [24586, 61464, 4610];
    let mut intp = Interpreter::new();
    intp.load_from_bytes(&test_instructions);
    intp.print_memory();
    intp.run();
}
