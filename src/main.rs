use std::{
    env,
    io::{self, Write},
};

use mra_parser::{parse_registers, RegisterDesc};

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
        if input.len() == 0 {
            break;
        }

        let it = data
            .range(String::from(&input)..)
            .take_while(|x| x.0.starts_with(&input));

        let m: Vec<&RegisterDesc> = it.map(|(_, v)| v).collect();

        if m.is_empty() {
            continue;
        }

        let mut index = 0;

        if m.len() != 1 {
            for (i, reg) in m.iter().enumerate() {
                println!("{}: {}", i, reg.name);
            }

            print!("{}> ", input);
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            index = match input.trim().parse::<usize>() {
                Ok(x) if x <= m.len() => x,
                _ => continue,
            }
        }

        println!("{}", m[index]);
    }
}
