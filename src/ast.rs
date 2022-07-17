pub struct Register<'a> {
    pub bits: u32,
    pub name: &'a str,
    pub bits_desc: Vec<Bitfield<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Bitfield<'a> {
    pub from: u32,
    pub to: u32,
    pub name: &'a str,
}
