#[cfg(test)]
use crate::ast::{Bitfield, Range, Register, Statement};

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub registers); // syntesized by LALRPOP
mod ast;

#[cfg(test)]
fn check_register(input: &str, reference: Register) {
    let stmt = registers::StatementParser::new().parse(input).unwrap();

    if let Statement::Register(reg) = stmt {
        assert_eq!(reg, reference);
    } else {
        panic!("Statement is not a Register")
    }
}

#[cfg(test)]
fn check_comment(input: &str) {
    let stmt = registers::StatementParser::new().parse(input).unwrap();
    assert!(matches!(stmt, Statement::Comment));
}

#[test]
fn register() {
    let input = "__register 32 {} SOME_REG;";
    check_register(
        input,
        Register {
            name: "SOME_REG",
            bits: 32,
            array: None,
            bits_desc: vec![],
        },
    );

    let input = "__register 32 { 31:31 OneBit, 15:0 SomeBits } ANOTHER_REG;";
    check_register(
        input,
        Register {
            name: "ANOTHER_REG",
            bits: 32,
            array: None,
            bits_desc: vec![
                Bitfield {
                    to: 31,
                    from: 31,
                    name: "OneBit",
                },
                Bitfield {
                    to: 15,
                    from: 0,
                    name: "SomeBits",
                },
            ],
        },
    );

    let input = "__register 32 { 31:31, 15:0 SomeBits } ANOTHER_REG;";
    check_register(
        input,
        Register {
            name: "ANOTHER_REG",
            bits: 32,
            array: None,
            bits_desc: vec![
                Bitfield {
                    to: 31,
                    from: 31,
                    name: "",
                },
                Bitfield {
                    to: 15,
                    from: 0,
                    name: "SomeBits",
                },
            ],
        },
    );

    let input = "__register 32 {  } EMPTY;";
    check_register(
        input,
        Register {
            name: "EMPTY",
            bits: 32,
            array: None,
            bits_desc: vec![],
        },
    );

    let input = "__register 32 { 1:1 A } REG;";
    check_register(
        input,
        Register {
            name: "REG",
            bits: 32,
            array: None,
            bits_desc: vec![Bitfield {
                to: 1,
                from: 1,
                name: "A",
            }],
        },
    );

    let input = "array [0..3] of __register 32 {  } ARRAY_REG;";
    check_register(
        input,
        Register {
            name: "ARRAY_REG",
            bits: 32,
            bits_desc: vec![],
            array: Some(Range { from: 0, to: 3 }),
        },
    );
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
