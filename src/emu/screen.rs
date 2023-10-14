#[derive(Debug, Clone)]
pub struct Screen {
    pub data: [bool; 64*32]
}

impl Screen {
    pub fn new() -> Self {
        Self {
            data: [false; 64*32]
        }
    }

    pub fn export(&self) -> Vec<u8> {
        let mut scr = Vec::with_capacity(64*32*4);
       
        for i in self.data.iter() {
            if *i {
                scr.push(0xab);
                scr.push(0xb2);
                scr.push(0xbf);
                scr.push(0xff);
            } else {
                scr.push(0x28);
                scr.push(0x2c);
                scr.push(0x34);
                scr.push(0xff);
            }
        }

        scr
    }
}
