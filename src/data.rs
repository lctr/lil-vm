#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reg(pub u8);
impl Reg {
    pub fn byte(&self) -> u8 {
        self.0
    }
}
impl std::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        u8::fmt(&self.0, f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Int(pub i32);

impl Int {
    pub fn bytes(&self) -> [u8; 2] {
        let a = self.0 as u16;
        let x = a;
        let y = a >> 8;
        [x as u8, y as u8]
    }
}

impl std::fmt::Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        i32::fmt(&self.0, f)
    }
}
