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
    assert_eq!(reg.name, "SOME_REG");
    assert_eq!(reg.bits, 32);

    // Missing ;
    assert!(registers::RegisterParser::new()
        .parse("__register 32 {} SOME_REG")
        .is_err());

    let reg = registers::RegisterParser::new()
        .parse("__register 32 { 31:31 OneBit, 15:0 SomeBits } ANOTHER_REG;")
        .unwrap();
    assert_eq!(reg.name, "ANOTHER_REG");
    assert_eq!(reg.bits, 32);
    assert_eq!(reg.bits_desc.len(), 2);
    assert_eq!(
        reg.bits_desc[0],
        Bitfield {
            to: 31,
            from: 31,
            name: "OneBit",
        }
    );
    assert_eq!(
        reg.bits_desc[1],
        Bitfield {
            to: 15,
            from: 0,
            name: "SomeBits",
        }
    );

    let reg = registers::RegisterParser::new()
        .parse("__register 32 {  } EMPTY;")
        .unwrap();
    assert_eq!(reg.name, "EMPTY");
    assert_eq!(reg.bits, 32);
    assert_eq!(reg.bits_desc.len(), 0);
}

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
