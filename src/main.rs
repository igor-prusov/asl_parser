use std::{env, fs};

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub registers); // syntesized by LALRPOP
mod ast;

fn main() {
    let f = env::args().nth(1).expect("No register file specified");
    println!("arg = {}", f);
    let input = fs::read_to_string(f).expect("Can't open file");
    let parser = registers::ProgramParser::new();
    let _program = parser.parse(&input).unwrap();
}
