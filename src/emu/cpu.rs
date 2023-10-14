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
        CpuOutput {
            should_beep: self.sound != 0
        }
    }
}
