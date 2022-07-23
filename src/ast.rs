pub enum Statement<'a> {
    Register(Register<'a>),
    Comment,
}

#[derive(Debug, PartialEq)]
pub struct Register<'a> {
    pub bits: u32,
    pub name: &'a str,
    pub bits_desc: Vec<Bitfield<'a>>,
    pub array: Option<Range>,
}

#[derive(Debug, PartialEq)]
pub struct Bitfield<'a> {
    pub from: u32,
    pub to: u32,
    pub name: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct Range {
    pub from: u32,
    pub to: u32,
}
