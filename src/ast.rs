pub struct Register {
    pub bits: u32,
    pub name: String,
    pub bits_desc: Vec<Bitfield>,
}

#[derive(Debug, PartialEq)]
pub struct Bitfield {
    pub from: u32,
    pub to: u32,
    pub name: String,
}
