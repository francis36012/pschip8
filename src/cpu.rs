extern crate rand;

use self::rand::ThreadRng;
use self::rand::Rng;

pub struct Cpu {
    pub registers: Reg,
    rng: ThreadRng,
}

impl Cpu {

    pub fn init() -> Self {
        Cpu {
            registers: Reg::default(),
            rng: rand::thread_rng(),
        }
    }
    /// Generates a random byte
    pub fn random_byte(&mut self) -> u8 {
        self.rng.gen_range(::std::u8::MIN, ::std::u8::MAX)
    }
}

#[derive(Default)]
pub struct Reg {
    pub v0: u8, pub v1: u8, pub v2: u8, pub v3: u8,
    pub v4: u8, pub v5: u8, pub v6: u8, pub v7: u8,
    pub v8: u8, pub v9: u8, pub va: u8, pub vb: u8,
    pub vc: u8, pub vd: u8, pub ve: u8, pub vf: u8,
    pub i: u16, pub pc: u16, pub sp: u8,
}

impl Reg {
    #[inline]
    pub fn get(&self, idx: u8) -> Option<u8> {
        match idx {
            0x0 =>  {Some(self.v0)},
            0x1 =>  {Some(self.v1)},
            0x2 =>  {Some(self.v2)},
            0x3 =>  {Some(self.v3)},
            0x4 =>  {Some(self.v4)},
            0x5 =>  {Some(self.v5)},
            0x6 =>  {Some(self.v6)},
            0x7 =>  {Some(self.v7)},
            0x8 =>  {Some(self.v8)},
            0x9 =>  {Some(self.v9)},
            0xa =>  {Some(self.va)},
            0xb =>  {Some(self.vb)},
            0xc =>  {Some(self.vc)},
            0xd =>  {Some(self.vd)},
            0xe =>  {Some(self.ve)},
            0xf =>  {Some(self.vf)},
            _ => {None},
        }
    }

    #[inline]
    pub fn set(&mut self, idx: u8, value: u8) {
        match idx {
            0x0 =>  {self.v0 = value},
            0x1 =>  {self.v1 = value},
            0x2 =>  {self.v2 = value},
            0x3 =>  {self.v3 = value},
            0x4 =>  {self.v4 = value},
            0x5 =>  {self.v5 = value},
            0x6 =>  {self.v6 = value},
            0x7 =>  {self.v7 = value},
            0x8 =>  {self.v8 = value},
            0x9 =>  {self.v9 = value},
            0xa =>  {self.va = value},
            0xb =>  {self.vb = value},
            0xc =>  {self.vc = value},
            0xd =>  {self.vd = value},
            0xe =>  {self.ve = value},
            0xf =>  {self.vf = value},
            _ => {},
        }
    }
}
