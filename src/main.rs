use std::{
    env,
    io::{self, Write},
};

use mra_parser::parse_registers;

fn main() {
    let f = env::args().nth(1).expect("No register file specified");
    println!("arg = {}", f);

    let data = parse_registers(&f);

    println!("Enter register names:");
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim().to_lowercase();

        match data.get(&input) {
            Some(reg) => println!("{}", reg),
            None => println!(""),
        }

        if input.len() == 0 {
            break;
        }
    }
}
