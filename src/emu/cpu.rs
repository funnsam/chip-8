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
    pub screen: &'a mut Screen
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
            0x3 => {},
            0x4 => {},
            0x5 => {},
            0x6 => { // set reg
                let x = nibble!(inst, 2);
                self.regs[x as usize] = inst as u8 & 0xFF;
            },
            0x7 => { // add imm
                let x = nibble!(inst, 2);
                self.regs[x as usize] += inst as u8 & 0xFF;
            },
            0x8 => {},
            0x9 => {},
            0xA => { // set i
                self.i = inst & 0xFFF;
            },
            0xB => {},
            0xC => {},
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
            0xE => {},
            0xF => {},
            _ => unreachable!(),
        }

        returns!()
    }
}
