use crate::ast::{Bitfield, Register, Statement};

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub registers); // syntesized by LALRPOP
mod ast;

#[cfg(test)]
fn check_register(input: &str, f: fn(Register)) {
    let stmt = registers::StatementParser::new().parse(input).unwrap();

    if let Statement::Register(reg) = stmt {
        f(reg);
    } else {
        panic!("Statement is not a Register")
    }
}

#[cfg(test)]
fn check_comment(input: &str) {
    let stmt = registers::StatementParser::new().parse(input).unwrap();

    if let Statement::Comment = stmt {
    } else {
        panic!("Statement is not a Register")
    }
}

#[test]
fn register() {
    let input = "__register 32 {} SOME_REG;";
    check_register(input, |reg| {
        assert_eq!(reg.name, "SOME_REG");
        assert_eq!(reg.bits, 32);
    });

    let input = "__register 32 { 31:31 OneBit, 15:0 SomeBits } ANOTHER_REG;";
    check_register(input, |reg| {
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
    });

    let input = "__register 32 {  } EMPTY;";
    check_register(input, |reg| {
        assert_eq!(reg.name, "EMPTY");
        assert_eq!(reg.bits, 32);
        assert_eq!(reg.bits_desc.len(), 0);
    });
}

#[test]
fn comment() {
    let input = "// some comment";
    check_comment(input);

    let input = "//some comment";
    check_comment(input);

    let input = "//some comment;";
    check_comment(input);

    let input = "///some comment";
    check_comment(input);
}

#[test]
fn bad_register() {
    // Missing ;
    assert!(registers::StatementParser::new()
        .parse("__register 32 {} SOME_REG")
        .is_err());
}

#[test]
fn program() {
    let input = "\
    __register 32 {} SOME_REG;\n
    __register 32 {} ANOTHER_REG;
    ";
    let prog = registers::ProgramParser::new().parse(input).unwrap();
    assert_eq!(prog.len(), 2);
    assert!(matches!(&prog[0], Statement::Register(_)));
    assert!(matches!(&prog[1], Statement::Register(_)));

    let input = "\
    __register 32 {} SOME_REG; __register 32 {} ANOTHER_REG;
    ";
    let prog = registers::ProgramParser::new().parse(input).unwrap();
    assert_eq!(prog.len(), 2);
    assert!(matches!(&prog[0], Statement::Register(_)));
    assert!(matches!(&prog[1], Statement::Register(_)));

    let input = "\
    // Some comment
    __register 32 {} REG;
    ";
    let prog = registers::ProgramParser::new().parse(input).unwrap();
    assert_eq!(prog.len(), 2);
    assert!(matches!(prog[0], Statement::Comment));
    assert!(matches!(&prog[1], Statement::Register(_)));
}

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
