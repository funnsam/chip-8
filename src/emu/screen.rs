#[derive(Debug, Clone)]
pub struct Screen {
    pub data: [u8; 64*32 / 8]
}
