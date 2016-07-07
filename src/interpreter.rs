extern crate sdl2;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};
use cpu::Cpu;

use self::sdl2::render::Renderer;
use self::sdl2::{VideoSubsystem, Sdl};
use self::sdl2::audio::{AudioDevice, AudioCallback, AudioSpecDesired};
use self::sdl2::pixels::Color;
use self::sdl2::rect::Point;

/// # Instructions Quick Reference
/// * 0nnn - SYS addr:       =>   jmp to nnn
/// * 00e0 - CLS:            =>   cls screen
/// * 00ee - RET:            =>   ret from subroutine, sp = sp - 1
/// * 1nnn - JP addr:        =>   jmp to nnn
/// * 2nnn - CALL addr:      =>   call subroutine at nnn, sp = sp + 1
/// * 3xkk - SE Vx, byte:    =>   skip next instruction if Vx = kk, pc = pc + 2
/// * 4xyk - SNE Vx, byte:   =>   skip next instruction if Vx != kk, pc = pc + 2
/// * 5xy0 - SE Vx, Vy:      =>   skip next instruction if Vx = Vy, pc = pc + 2
/// * 6xkk - LD Vx, byte:    =>   set Vx = kk
/// * 7xkk - ADD Vx, byte:   =>   set Vx = Vx + kk
/// * 8xy0 - LD Vx, Vy:      =>   set Vx = Vy
/// * 8xy1 - OR Vx, Vy:      =>   set Vx = Vx OR Vy
/// * 8xy2 - AND Vx, Vy:     =>   set Vx = Vx AND Vy
/// * 8xy3 - XOR Vx, Vy:     =>   set Vx = Vx XOR Vy
/// * 8xy4 - ADD Vx, Vy:     =>   set Vx = Vx + Vy, VF = 1 (iff result > 255) : 0
/// * 8xy5 - SUB Vx, Vy:     =>   set Vx = Vx - Vy, VF = 1 (iff Vx > Vy) : 0
/// * 8xy6 - SHR Vx {, Vy}:  =>   set Vx = Vx SHR 1, VF = 1 (iff lsb(Vx) = 1) : 0
/// * 8xy7 - SUBN Vx, Vy:    =>   set Vx = Vy - Vx, VF = 1 (iff Vy > Vx) : 0
/// * 8xye - SHL Vx {, Vy}:  =>   set Vx = Vx SHL 1, VF = 1 (iff msb(Vx) = 1) : 0
/// * 9xy0 - SNE Vx, Vy:     =>   skip next instruction if Vx != Vy
/// * Annn - LD I, addr:     =>   set I = nnn
/// * Bnnn - JP V0, addr:    =>   jmp to nnn + V0, pc = nnn + v0
/// * Cxkk - RND Vx, byte:   =>   set Vx = random byte AND kk
/// * Dxyn - DRW Vx, Vy, n:  =>   display n-byte sprite starting at addr I at (Vx, Vy)
///                          =>   VF = 1, if after anything on screen is erased
/// * Ex9e - SKP Vx:         =>   skip next instruction if key with value of Vx is pressed
/// * Exa1 - SKNP Vx:        =>   skip next instruction if key with value of Vx is not pressed
/// * Fx07 - LD Vx, DT:      =>   set Vx = delay timer value
/// * Fx0a - LD Vx, K:       =>   wait for input and store the value of the key press in Vx
/// * Fx15 - LD DT, Vx:      =>   set the value of the delay timer to be the value of Vx
/// * Fx18 - LD ST, Vx:      =>   set the the value of the sound timer to be the value of Vx
/// * Fx1e - ADD I, Vx:      =>   set I = I + Vx
/// * Fx29 - LD F, Vx:       =>   set I = location of sprite (font) for digit Vx
/// * Fx33 - LD B, Vx:       =>   store BCD representation of Vx in mem addresses I, I+1 and I+2
/// * Fx55 - LD [I], Vx:     =>   store values of registers V0 through Vx at address starting at I
/// * Fx65 - LD Vx, [I]:     =>   starting from memory address I, populate registers V0 to Vx


const INTERPRETER_END: u16 = 512;
const FONT_SPRITES_MEM_START: u16 = 0;
const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;
const MEMORY_SIZE: u16 = 4096;
const STACK_DEPTH: u8 = 16;
const KEYS_N: u8 = 16;
const INSTRUCTION_WIDTH: u8 = 2;
const MAX_SPRITE_LENGTH: u8 = 15;

static DEFAULT_WINDOW_TITLE: &'static str = "pschip8";
const DEFAULT_VIDEO_SCALE: u8 = 4;

const FONT_SPRITES: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // "0"
    0x20, 0x60, 0x20, 0x20, 0x70, // "1"
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // "2"
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // "3"
    0x90, 0x90, 0xf0, 0x10, 0x10, // "4"
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // "5"
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // "6"
    0xf0, 0x10, 0x20, 0x40, 0x40, // "7"
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // "8"
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // "9"
    0xf0, 0x90, 0xf0, 0x90, 0x90, // "A"
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // "B"
    0xf0, 0x80, 0x80, 0x80, 0xf0, // "C"
    0xe0, 0x90, 0x90, 0x90, 0xe0, // "D"
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // "E"
    0xf0, 0x80, 0xf0, 0x80, 0x80, // "F"
];

static DESIRED_AUDIO_SPEC: AudioSpecDesired = AudioSpecDesired {
    freq: Some(44100),
    channels: Some(1),
    samples: Some(2048),
};

struct VideoSystem<'a> {
    width: u8,
    height: u8,
    scale_factor: u8,
    memory: Vec<bool>,
    renderer: Renderer<'a>,
}

impl <'a> VideoSystem<'a> {
    fn default(video_sys: &VideoSubsystem) -> Self {
        let window = video_sys.window(DEFAULT_WINDOW_TITLE,
                            SCREEN_WIDTH as u32 * DEFAULT_VIDEO_SCALE as u32,
                            SCREEN_HEIGHT as u32 * DEFAULT_VIDEO_SCALE as u32).build().unwrap();

        VideoSystem {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            scale_factor: DEFAULT_VIDEO_SCALE,
            memory: vec![false; ((SCREEN_WIDTH as usize) * (SCREEN_HEIGHT as usize))],
            renderer: window.renderer().present_vsync().build().unwrap(),
        }
    }

    fn draw(&mut self, x: u8, y: u8, sprite: &[u8]) -> bool {
        let mut erased = false;
        let sprite_len = sprite.len();

        if (x >= self.width) || (y >= self.height) || (sprite_len as u8 > MAX_SPRITE_LENGTH) {
            return erased;
        }
        let mut i = y;
        while (i - y) < sprite_len as u8 && (i < self.height) {
            let start = i as usize * self.width as usize + x as usize;
            let vidlim = i as usize * self.width as usize + self.width as usize;

            let mut j = start;
            while (j < start + 8) && (j < vidlim) {
                let shifts = (8 - (j - start)) - 1;
                let prev = self.memory[j as usize];
                let new = ((sprite[(i - y) as usize] >> shifts) & 0x1) == 1;
                self.memory[j] = prev != new;
                erased = if !erased { (new && prev) || (!new && prev) } else { erased };
                j += 1;
            }
            i += 1
        }
        erased
    }

    fn render_screen(&mut self) {
        self.renderer.set_scale(self.scale_factor as f32, self.scale_factor as f32);
        self.renderer.set_draw_color(Color::RGB(0, 0, 0));
        self.renderer.clear();

        for (index, pixel) in self.memory.iter().enumerate() {
            let y = index / self.width as usize;
            let x = index - (y * self.width as usize);

            let color = if *pixel {
                Color::RGB(255, 255, 255)
            } else {
                Color::RGB(0, 0, 0)
            };
            self.renderer.set_draw_color(color);
            self.renderer.draw_point(Point::new(x as i32, y as i32));
        }
        self.renderer.present();
    }

    fn clear_screen(&mut self) {
        for idx in 0..self.memory.len() {
            self.memory[idx] = false;
        }
    }
}

struct Tone {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for Tone {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = match self.phase {
                0.0...0.5 => self.volume,
                _ => -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

struct SoundSystem {
    au_dev: AudioDevice<Tone>,
}

impl SoundSystem {
    fn resume(&self) {
        self.au_dev.resume();
    }

    fn pause(&self) {
        self.au_dev.pause();
    }

    fn new(au_dev: AudioDevice<Tone>) -> Self {
        SoundSystem {
            au_dev: au_dev
        }
    }
}
pub struct Interpreter<'a> {
    cpu: Cpu,
    memory: [u8; MEMORY_SIZE as usize],
    stack: [u16; STACK_DEPTH as usize],
    delay_timer: u8,
    sound_timer: u8,
    sdl: Sdl,
    sound_system: SoundSystem,
    video_system: VideoSystem<'a>,
    key: [u8; KEYS_N as usize],
}

impl <'a> Interpreter<'a> {
    pub fn new() -> Interpreter<'a> {
        let sdl_ctxt = sdl2::init().unwrap();
        let au_sys = sdl_ctxt.audio().unwrap();
        let vd_sys = sdl_ctxt.video().unwrap();

        let mut interpreter = Interpreter {
            cpu: Cpu::init(),
            memory: [0; 4096],
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            sdl: sdl_ctxt,
            sound_system: SoundSystem::new(au_sys.open_playback(None, &DESIRED_AUDIO_SPEC, |spec| {
                Tone {
                    phase_inc: 440.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.5,
                }
            }).unwrap()),
            video_system: VideoSystem::default(&vd_sys),
            key: [0; 16],
        };
        for i in FONT_SPRITES_MEM_START..(FONT_SPRITES_MEM_START + FONT_SPRITES.len() as u16) {
            interpreter.memory[i as usize] = FONT_SPRITES[(i - FONT_SPRITES_MEM_START) as usize];
        }
        interpreter
    }

    pub fn load_program_from_file(&mut self, path: &Path) {
        let mut file = File::open(path).unwrap();
        let mut mem_idx = INTERPRETER_END as usize;
        let mut buf = [0 as u8; 2];
        loop {
            let rc = file.read(&mut buf).unwrap();
            if (mem_idx >= self.memory.len() - 1) || rc != 2 {
                break;
            }
            self.memory[mem_idx] = buf[0];
            self.memory[mem_idx + 1] = buf[1];
            mem_idx += 2;
        }
        self.cpu.registers.pc = INTERPRETER_END;
    }

    pub fn load_from_bytes(&mut self, instructions: &[u16]) {
        if instructions.len() > (MEMORY_SIZE - INTERPRETER_END) as usize {
            panic!();
        }
        let mut mem_idx = INTERPRETER_END as usize;
        for instruction in instructions {
            self.memory[mem_idx] = ((instruction >> 8) & 0x00ffu16) as u8;
            self.memory[mem_idx + 1] = (instruction & 0x00ffu16) as u8;
            mem_idx += 2;
        }
        self.cpu.registers.pc = INTERPRETER_END;
    }

    pub fn print_memory(&self) {
        println!("Memory:");
        let mut lidx = 0;

        loop {
            print!("{}: {}", lidx, self.memory[lidx]);
            lidx += 1;
            if lidx >= MEMORY_SIZE as usize {
                break;
            } else {
                print!(", ");
            }
            if lidx >= 8 && lidx % 8 == 0 {
                println!("");
            }
        }
        println!("\n");
    }

    pub fn print_registers(&self) {
        println!("Registers:");
        println!("V0: {}, V1: {}, V2: {}, V3: {}",
               self.cpu.registers.v0, self.cpu.registers.v1,
               self.cpu.registers.v2, self.cpu.registers.v3);
        println!("V4: {}, V5: {}, V6: {}, V7: {}",
               self.cpu.registers.v4, self.cpu.registers.v5,
               self.cpu.registers.v6, self.cpu.registers.v7);
        println!("V8: {}, V9: {}, Va: {}, Vb: {}",
               self.cpu.registers.v9, self.cpu.registers.v9,
               self.cpu.registers.va, self.cpu.registers.vb);
        println!("Vc: {}, Vd: {}, Ve: {}, Vf: {}",
               self.cpu.registers.vc, self.cpu.registers.vd,
               self.cpu.registers.ve, self.cpu.registers.vf);
        println!("i: {}, pc: {}, sp: {}",
                 self.cpu.registers.i, self.cpu.registers.pc,
                 self.cpu.registers.sp);
        println!("");
    }

    fn cycle(&mut self) {
        let instruction = self.fetch();
        let opcode = ((instruction & 0xf000u16) >> 12) as u8;

        println!("[DEBUG]  About to execute: 0x{:x}", instruction);
        self.print_registers();

        match opcode {
            0x0 => {
                let lnnn = instruction & 0x0fffu16;
                // clear screen
                if lnnn == 0x00e0 {
                    self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
                    self.video_system.clear_screen();

                // return from subroutine
                } else if lnnn == 0x00ee {
                    self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
                    let sp = match self.cpu.registers.sp {
                        0 => 0,
                        n @ 1...15 => {
                            self.cpu.registers.sp -= 1;
                            n - 1
                        },
                        _ => 0
                    };
                    self.cpu.registers.pc = self.stack[sp as usize];
                }
            },
            0x1 => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;

                // jmp nnn
                let nnn = instruction & 0x0fff;
                self.cpu.registers.pc = nnn;
            },
            0x2 => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;

                // CALL addr:nnn
                let nnn = instruction & 0x0fff;
                self.stack[self.cpu.registers.sp as usize] = self.cpu.registers.pc;
                self.cpu.registers.sp +=  1 % (STACK_DEPTH - 1);
                self.cpu.registers.pc = nnn;
            },
            0x3 => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;

                // 3xkk - SE Vx, byte
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;

                let vx = self.cpu.registers.get(x).unwrap();
                if vx == kk { self.cpu.registers.pc += INSTRUCTION_WIDTH as u16 }
            },
            0x4 => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
                // 4xkk - SNE Vx, byte
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;

                let vx = self.cpu.registers.get(x).unwrap();
                if vx != kk { self.cpu.registers.pc += INSTRUCTION_WIDTH as u16 }
            },
            0x5 => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
                // 4xy0 - SE Vx, Vy
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let y = (instruction >> 4u16 & 0x000fu16) as u8;
                let vx = self.cpu.registers.get(x).unwrap();
                let vy = self.cpu.registers.get(y).unwrap();
                if vx == vy { self.cpu.registers.pc += INSTRUCTION_WIDTH as u16 }
            },
            0x6 => {
                // 6xkk - LD Vx, byte
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;
                self.cpu.registers.set(x, kk);
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
            },
            0x7 => {
                // 7xkk - ADD Vx, byte
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as usize;
                let vx = self.cpu.registers.get(x).unwrap() as usize;

                // check if addition causes u8 overflow
                if (vx + kk) > 255 {
                    self.cpu.registers.vf = 1;
                } else {
                    self.cpu.registers.vf = 0;
                }
                self.cpu.registers.set(x, (vx + kk) as u8);
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
            },
            0x8 => {
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let y = (instruction >> 4u16 & 0x000fu16) as u8;
                let n = (instruction & 0x000fu16) as u8;

                match n {
                    // 8xy0 - LD Vx, Vy
                    0x0 => {
                        let vy = self.cpu.registers.get(y).unwrap();
                        self.cpu.registers.set(x, vy);
                    },
                    // 8xy1 - OR Vx, Vy
                    0x1 => {
                        let vx = self.cpu.registers.get(x).unwrap();
                        let vy = self.cpu.registers.get(y).unwrap();
                        self.cpu.registers.set(x, vx | vy);
                    },
                    // 8xy2 - AND Vx, Vy
                    0x2 => {
                        let vx = self.cpu.registers.get(x).unwrap();
                        let vy = self.cpu.registers.get(y).unwrap();
                        self.cpu.registers.set(x, vx & vy);
                    },
                    // 8xy3 - XOR Vx, Vy
                    0x3 => {
                        let vx = self.cpu.registers.get(x).unwrap();
                        let vy = self.cpu.registers.get(y).unwrap();
                        self.cpu.registers.set(x, vx ^ vy);
                    },
                    // 8xy4 - ADD Vx, Vy
                    0x4 => {
                        let vx = self.cpu.registers.get(x).unwrap() as usize;
                        let vy = self.cpu.registers.get(y).unwrap() as usize;

                        if (vx + vy) > 255 {
                            self.cpu.registers.vf = 1;
                        } else {
                            self.cpu.registers.vf = 0;
                        }
                        self.cpu.registers.set(x, (vx + vy) as u8);
                    },
                    // 8xy5 - SUB Vx, Vy
                    0x5 => {
                        let vx = self.cpu.registers.get(x).unwrap() as usize;
                        let vy = self.cpu.registers.get(y).unwrap() as usize;

                        if vx > vy {
                            self.cpu.registers.vf = 1;
                            self.cpu.registers.set(x, (vx - vy) as u8);
                        } else {
                            self.cpu.registers.vf = 0;
                            self.cpu.registers.set(x, 0);
                        }
                    },
                    // 8xy6 - SHR Vx {, Vy}
                    0x6 => {
                        let vx = self.cpu.registers.get(x).unwrap();
                        if (vx & 0x01) == 0x01 {
                            self.cpu.registers.vf = 1;
                        } else {
                            self.cpu.registers.vf = 0;
                        }
                        self.cpu.registers.set(x, ((vx as usize) >> 1) as u8);
                    },
                    // 8xy7 - SUBN Vx ,Vy
                    0x7 => {
                        let vx = self.cpu.registers.get(x).unwrap() as usize;
                        let vy = self.cpu.registers.get(y).unwrap() as usize;

                        if vx < vy {
                            self.cpu.registers.vf = 1;
                            self.cpu.registers.set(x, (vy - vx) as u8);
                        } else {
                            self.cpu.registers.vf = 0;
                            self.cpu.registers.set(x, 0);
                        }
                    },
                    // 8xy6 - SHL Vx {, Vy}
                    0xe => {
                        let vx = self.cpu.registers.get(x).unwrap();
                        if (vx & 0x10) == 0x10 {
                            self.cpu.registers.vf = 1;
                        } else {
                            self.cpu.registers.vf = 0;
                        }
                        self.cpu.registers.set(x, ((vx as usize) << 1) as u8);
                    },
                    _ => { }
                }
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
            },
            0x9 => {
                // 9xy0 - SNE Vx, Vy
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let y = (instruction >> 4u16 & 0x000fu16) as u8;
                let vx = self.cpu.registers.get(x).unwrap();
                let vy = self.cpu.registers.get(y).unwrap();
                if vx != vy { self.cpu.registers.pc += INSTRUCTION_WIDTH as u16 }
            },
            0xa => {
                // Annn - LD I, addr
                let nnn = instruction & 0x0fff;
                self.cpu.registers.i = nnn;
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
            },
            0xb => {
                // Bnnn - JP V0, addr
                let nnn = instruction & 0x0fff;
                let v0 = self.cpu.registers.v0;
                self.cpu.registers.pc = nnn + (v0 as u16);
            },
            0xc => {
                // Cxkk - RND Vx, byte
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;
                let random = self.cpu.random_byte();
                self.cpu.registers.set(x, random & kk);
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
            },
            0xd => {
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;
                // Dxyn - DRW Vx, Vy, nibble
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let y = (instruction >> 4u16 & 0x000fu16) as u8;
                let n = instruction & 0x000fu16;
                let i = self.cpu.registers.i;
                let sprite = &self.memory[(i as usize..(i+n) as usize)];
                let erased = self.video_system.draw(self.cpu.registers.get(x).unwrap_or(0), self.cpu.registers.get(y).unwrap_or(0), sprite);
                self.cpu.registers.vf = if erased { 1 } else { 0 };
            },
            0xe => {
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;

                match kk {
                    // Ex9e - SKP Vx
                    0x9e => {
                        // TODO: Skip if value of key pressed equals Vx
                    },
                    // Exa1 - SKNP Vx
                    0xa1 => {
                        // TODO: Skip if value of key pressed does not equal value of Vx
                    },
                    _ => { }
                }
            },
            0xf => {
                let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                let kk = (instruction & 0x00ffu16) as u8;
                self.cpu.registers.pc += INSTRUCTION_WIDTH as u16;

                match kk {
                    // Fx07 - LD Vx, DT
                    0x07 => {
                        self.cpu.registers.set(x, self.delay_timer);
                    },
                    // Fx0a - LD Vx, K
                    0x0a => {
                        // TODO: wait for key press and store it in Vx
                    },
                    // Fx15 - LD  DT, Vx
                    0x15 => {
                        self.delay_timer = self.cpu.registers.get(x).unwrap();
                    },
                    // Fx18 - LD ST, Vx
                    0x18 => {
                        self.sound_timer = self.cpu.registers.get(x).unwrap();
                    },
                    // Fx1e - ADD I, Vx
                    0x1e => {
                        let regv = self.cpu.registers.get(x).unwrap();
                        self.cpu.registers.i += regv as u16;
                    },
                    // Fx29 - LD F, Vx
                    0x29 => {
                        let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                        let vx = self.cpu.registers.get(x).unwrap() as u16;
                        if vx <= 0xf {
                            self.cpu.registers.i = FONT_SPRITES_MEM_START + (vx as u16 * 5);
                        }
                    },
                    // Fx33 - LD B, Vx
                    0x33 => {
                        let x = ((instruction >> 8u16) & 0x000fu16) as u8;
                        let vx = self.cpu.registers.get(x).unwrap();
                        let vx_bcd = u8_to_bcd(vx);
                        let ireg = self.cpu.registers.i as usize;

                        for i in 0..3 {
                            self.memory[ireg + i] = vx_bcd[i];
                        }
                    },
                    // Fx55 - LD [I], Vx
                    0x55 => {
                        let ireg = self.cpu.registers.i;
                        for i in 0..(x + 1) {
                            let regv = self.cpu.registers.get(i).unwrap();
                            self.memory[(ireg + i as u16) as usize] = regv;
                        }
                    },
                    // Fx65 - LD Vx, [I]
                    0x65 => {
                        let ireg = self.cpu.registers.i;
                        for i in 0..(x + 1) {
                            let mem_val = self.memory[(ireg + i as u16) as usize];
                            self.cpu.registers.set(i, mem_val as u8);
                        }
                    },
                    _ => { }
                }
            },
            _ => {
                // This should never happen
                panic!("The end is near!");
            }
        }
    }

    pub fn run(&mut self) {
        // nanoseconds per frame
        let spf_nano = Duration::new(0, 16_666_666);
        loop {
            let time_start = SystemTime::now();
            self.cycle();
            self.input_routine();
            self.timer_routine();
            self.video_system.render_screen();
            let elapsed = SystemTime::now().duration_since(time_start).unwrap();
            if elapsed < spf_nano {
                thread::sleep(spf_nano - elapsed);
            }
        }
    }

    /// Checks and updates the delay and sound timers when necessary.
    fn timer_routine(&mut self) {
        let sound_timer = self.sound_timer;
        if sound_timer > 0 {
            self.sound_timer -= 1;
            self.sound_system.resume();
        } else {
            self.sound_system.pause();
        }

        let delay_timer = self.delay_timer;
        if delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn input_routine(&mut self) {
    }

    fn fetch(&self) -> u16 {
        let pc = self.cpu.registers.pc as usize;
        ((self.memory[pc] as u16) << 8) | self.memory[(pc + 1) as usize] as u16
    }
}

/// Returns the binary decimal coding for the specified number
/// The hundreds, tens, and ones digits goes in the
/// first, second, and third positions respectively of the returned
/// three-element array.
fn u8_to_bcd(input: u8) -> [u8; 3] {
    let mut result: [u8; 3] = [0; 3];
    result[0] = input / 100;
    result[1] = (input - (result[0] * 100)) / 10;
    result[2] = input - ((result[0] * 100) + (result[1] * 10));

    result
}
