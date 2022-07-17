use crate::ast::Bitfield;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub registers); // syntesized by LALRPOP
mod ast;

#[test]
fn asl() {
    let reg = registers::RegisterParser::new()
        .parse("__register 32 {} SOME_REG;")
        .unwrap();
    assert_eq!(reg.as_ref().name, "SOME_REG");
    assert_eq!(reg.as_ref().bits, 32);

    // Missing ;
    assert!(registers::RegisterParser::new()
        .parse("__register 32 {} SOME_REG")
        .is_err());

    let reg = registers::RegisterParser::new()
        .parse("__register 32 { 31:31 OneBit, 15:0 SomeBits } ANOTHER_REG;")
        .unwrap();
    assert_eq!(reg.as_ref().name, "ANOTHER_REG");
    assert_eq!(reg.as_ref().bits, 32);
    assert_eq!(reg.as_ref().bits_desc.len(), 2);
    assert_eq!(
        *reg.as_ref().bits_desc[0].as_ref(),
        Bitfield {
            to: 31,
            from: 31,
            name: String::from("OneBit"),
        }
    );
    assert_eq!(
        *reg.as_ref().bits_desc[1].as_ref(),
        Bitfield {
            to: 15,
            from: 0,
            name: String::from("SomeBits"),
        }
    );

    let reg = registers::RegisterParser::new()
        .parse("__register 32 {  } EMPTY;")
        .unwrap();
    assert_eq!(reg.as_ref().name, "EMPTY");
    assert_eq!(reg.as_ref().bits, 32);
    assert_eq!(reg.as_ref().bits_desc.len(), 0);
}

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
