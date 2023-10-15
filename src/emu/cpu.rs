use super::*;

#[derive(Debug)]
pub struct CPU<'a> {
    pub pc    : u16,
    pub i     : u16,
    pub stack : Vec<u16>,
    pub delay : u8,
    pub sound : u8,
    pub regs  : [u8; 16],
    pub memory: [u8; 4096],
    pub screen: &'a mut Screen,
    pub random: u32,
}

#[derive(Debug)]
pub struct CpuInput {
    pub key_pressed: [bool; 16]
}

#[derive(Debug)]
pub struct CpuOutput {
    pub should_beep: bool,
}

impl<'a> CPU<'a> {
    pub fn new(screen: &'a mut Screen, code: Vec<u8>) -> Self {
        let mut mem = vec![0; 4096];
        mem.splice(0..FONT.len(), FONT.iter().cloned());
        mem.splice(512..code.len()+512, code.iter().cloned());
        Self {
            pc    : 512,
            i     : 0,
            stack : Vec::with_capacity(16),
            delay : 0,
            sound : 0,
            regs  : [0; 16],
            memory: mem.try_into().unwrap(),
            screen,
            random: 1,
        }
    }

    pub fn cycle(&mut self, input: CpuInput) -> CpuOutput {
        macro_rules! returns {
            () => {
                return CpuOutput {
                    should_beep: self.sound != 0
                }
            };
        }

        macro_rules! nibble {
            ($a: expr, $b: expr) => {{
                let a = $a;
                (a >> $b*4) & 0xF
            }};
        }

        // fetch
        let inst_hi = self.memory[self.pc as usize];
        let inst_lo = self.memory[self.pc as usize + 1];
        let inst = (inst_hi as u16) << 8 | inst_lo as u16;
        self.pc += 2;

        // decode / execute
        match nibble!(inst, 3) {
            0x0 => match inst & 0xFFF { // machine command
                0x0E0 => { // clear screen
                    for i in self.screen.data.iter_mut() {
                        *i = false
                    }
                },
                0x0EE => { // return
                    self.pc = self.stack.pop().unwrap();
                },
                _ => panic!("{inst:04x}"),
            },
            0x1 => { // jump
                self.pc = inst & 0xFFF;
            },
            0x2 => { // call
                self.stack.push(self.pc);
                self.pc = inst & 0xFFF;
            },
            0x3 => if self.regs[nibble!(inst, 2) as usize] == inst as u8 { self.pc += 2; }, // skip if eq
            0x4 => if self.regs[nibble!(inst, 2) as usize] != inst as u8 { self.pc += 2; }, // skip if ne
            0x5 => if self.regs[nibble!(inst, 2) as usize] == self.regs[nibble!(inst, 1) as usize] { self.pc += 2; }, // skip if eq
            0x6 => { // set reg
                let x = nibble!(inst, 2);
                self.regs[x as usize] = inst as u8 & 0xFF;
            },
            0x7 => { // add imm
                let x = nibble!(inst, 2);
                self.regs[x as usize] += inst as u8 & 0xFF;
            },
            0x8 => {
                let y = self.regs[nibble!(inst, 1) as usize];
                let x = &mut self.regs[nibble!(inst, 2) as usize];
                let (res, flg) = match nibble!(inst, 0) {
                    0x0 => (y, None), // set
                    0x1 => (*x | y, None), // or
                    0x2 => (*x & y, None), // and
                    0x3 => (*x ^ y, None), // xor
                    0x4 => (*x + y, Some(x.checked_add(y).is_none() as u8)), // add
                    0x5 => (*x - y, Some(x.checked_sub(y).is_some() as u8)), // sub
                    0x6 => (if true { y } else { *x } >> 1, Some(y & 1)), // TODO: make quirk
                                                                         // configable
                    0x7 => (y - *x, Some(y.checked_sub(*x).is_some() as u8)), // sub rev
                    0xE => (if true { y } else { *x } << 1, Some(y >> 7)), // lsh
                    _ => todo!("{inst:04x}"),
                };
                *x = res;
                if let Some(v) = flg {
                    self.regs[0xF] = v
                }
            },
            0x9 => if self.regs[nibble!(inst, 2) as usize] != self.regs[nibble!(inst, 1) as usize] { self.pc += 2; }, // skip if ne
            0xA => { // set i
                self.i = inst & 0xFFF;
            },
            0xB => { // jump offset
                let x = self.regs[if true { nibble!(inst, 2) as usize } else { 0 }]; // TODO: make
                                                                                     // quirk configable
                self.pc = (inst & 0xFFF) + x as u16;
            },
            0xC => {
                let x = nibble!(inst, 2);
                let mask = inst & 0xFF;

                // xorshift https://en.wikipedia.org/wiki/Xorshift#Example_implementation
                self.random ^= self.random << 13;
                self.random ^= self.random >> 17;
                self.random ^= self.random << 5;
                
                self.regs[x as usize] = self.random as u8 & mask as u8;
            },
            0xD => {
                let x = self.regs[nibble!(inst, 2) as usize] & 63;
                let mut y = self.regs[nibble!(inst, 1) as usize] & 31;
                let n = nibble!(inst, 0);
                self.regs[0xF] = 0;
                for i in 0..n as usize {
                    let mut spr = self.memory[self.i as usize + i];
                    let mut p = x as usize + (y as usize * 64);
                    for _ in 0..8 {
                        if spr & 0x80 > 0 {
                            self.screen.data[p] ^= true;
                            if !self.screen.data[p] {
                                self.regs[0xF] = 1;
                            }
                        }

                        spr <<= 1;
                        p += 1;
                    }

                    y += 1;
                    if y == 32 { break; }
                }
            },
            0xE => {
                let x = self.regs[nibble!(inst, 2) as usize];
                let wanted_state = inst & 1 == 0;
                if input.key_pressed[x as usize & 0xF] == wanted_state {
                    self.pc += 2;
                }
            },
            0xF => {
                let x = &mut self.regs[nibble!(inst, 2) as usize];
                match inst & 0xFF {
                    0x07 => *x = self.delay, // set x to delay
                    0x0A => { // wait for any key
                        for (ki, ks) in input.key_pressed.iter().enumerate() {
                            if *ks {
                                *x = ki as u8;
                                returns!();
                            }
                        }
                        self.pc -= 2;
                    },
                    0x15 => self.delay = *x, // set delay to x
                    0x18 => self.sound = *x, // set sound to x
                    0x1E => { // add i by x
                        self.i += *x as u16;
                        self.regs[0xF] = (self.i >> 12) as u8;
                    },
                    0x29 => self.i = *x as u16 * 5, // font char
                    0x33 => { // bcd
                        let x = self.regs[nibble!(inst, 2) as usize];
                        let d0 = x / 10 / 10 % 10;
                        let d1 = x / 10 % 10;
                        let d2 = x % 10;
                        self.memory[self.i as usize + 0] = d0;
                        self.memory[self.i as usize + 1] = d1;
                        self.memory[self.i as usize + 2] = d2;
                    },
                    0x55 => { // store
                        let x = nibble!(inst, 2) as usize;
                        for n in 0..=x {
                            self.memory[self.i as usize + n] = self.regs[n];
                            if false { self.i += 1; } // TODO: make configable
                        }
                    },
                    0x65 => { // load
                        let x = nibble!(inst, 2) as usize;
                        for n in 0..=x {
                            self.regs[n] = self.memory[self.i as usize + n];
                            if false { self.i += 1; } // TODO: make configable
                        }
                    },
                    _ => todo!("{inst:04x}")
                }
            },
            _ => unreachable!(),
        }

        returns!()
    }
}
