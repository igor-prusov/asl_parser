use crate::ast::Statement;
use core::fmt;
use std::{cmp::max, collections::BTreeMap};

#[cfg(test)]
use crate::ast::{Bitfield, Range, Register};

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

#[derive(Debug)]
pub struct BitfieldDesc {
    pub from: u32,
    pub to: u32,
    pub name: String,
}

#[derive(Debug)]
pub struct RegisterDesc {
    pub name: String,
    pub bits: u32,
    pub fields: Vec<BitfieldDesc>,
}

impl fmt::Display for RegisterDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut names = Vec::new();
        let mut ranges = Vec::new();
        enum Row {
            Names,
            Bits,
        }

        writeln!(f, "{}", self.name)?;

        for field in &self.fields {
            names.push(format! {" {} ", field.name});
            ranges.push(if field.from == field.to {
                format!(" {} ", field.to)
            } else {
                format!(" {}..{} ", field.to, field.from)
            });
        }

        let print_row = |f: &mut fmt::Formatter, index: Row| -> fmt::Result {
            for (a, b) in names.iter().zip(&ranges) {
                let size = max(a.len(), b.len());
                let c = match &index {
                    Row::Names => a,
                    Row::Bits => b,
                };

                let offset = (size - c.len()) / 2;
                write!(f, "|")?;
                for _ in 0..offset {
                    write!(f, " ")?;
                }
                write!(f, "{}", c)?;

                for _ in offset + c.len()..size {
                    write!(f, " ")?;
                }
            }

            writeln!(f, "|")
        };

        let print_line = |f: &mut fmt::Formatter| -> fmt::Result {
            for (a, b) in names.iter().zip(&ranges) {
                let size = max(a.len(), b.len());
                write!(f, "+")?;
                for _ in 0..size {
                    write!(f, "-")?;
                }
            }
            writeln!(f, "+")
        };

        print_line(f)?;
        print_row(f, Row::Bits)?;
        print_line(f)?;
        print_row(f, Row::Names)?;
        print_line(f)
    }
}

pub fn parse_registers(input: &str) -> BTreeMap<String, RegisterDesc> {
    let parser = registers::ProgramParser::new();
    let program = parser.parse(input).unwrap();

    let mut data = BTreeMap::new();

    for stmt in program {
        if let Statement::Register(reg) = stmt {
            let mut fields = Vec::new();
            let mut expected = Some(reg.bits - 1);

            /* Iterate over parsed Bitfields and add padding with anonymous BitfieldDescs */
            for f in reg.bits_desc {
                /* Add padding before bitfield */
                expected.filter(|x| *x != f.to).map(|x| {
                    fields.push(BitfieldDesc {
                        from: f.to + 1,
                        to: x,
                        name: String::new(),
                    })
                });

                fields.push(BitfieldDesc {
                    from: f.from,
                    to: f.to,
                    name: f.name.to_string(),
                });

                expected = f.from.checked_sub(1);
            }

            /* Add padding after bitfield */
            if let Some(x) = expected {
                fields.push(BitfieldDesc {
                    from: 0,
                    to: x,
                    name: String::new(),
                })
            }

            data.insert(
                reg.name.to_lowercase(),
                RegisterDesc {
                    name: reg.name.to_string(),
                    bits: reg.bits,
                    fields,
                },
            );
        }
    }

    data
}
