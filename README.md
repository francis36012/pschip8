# pschip8

This is a chip-8 interpreter written in the rust programming language. This
is also my first emulator (or interpreter) project.

# Screenshots
![breakout_1](https://github.com/francis36012/pschip8/raw/master/screenshots/breakout_1.png)
![breakout_2](https://github.com/francis36012/pschip8/raw/master/screenshots/breakout_2.png)

![c8-logo_1](https://github.com/francis36012/pschip8/raw/master/screenshots/chip8_logo_1.png)
![c8-logo_2](https://github.com/francis36012/pschip8/raw/master/screenshots/chip8_logo_2.png)

# Building and Install
`cargo` is used to build the interpreter

To build
```
git clone https://github.com/francis36012/pschip8
cd pschip8
cargo build --release
```
After building, using the command above, you will find the binary in target/release

You can install without building using cargo:
```
cargo install --git https://github.com/francis36012/pschip8
```

This will install the binary `pschip8` into the cargo install root directory
which is `~/.cargo/bin` by default.

# Usage
```shell
pschip8 -p <program-file>
```
