use std::{
    fs::File,
    io::{self, Read, Write},
};

use mra_parser::{parse_registers, RegisterDesc};

fn init_state() -> File {
    let mut regs_asl_path = dirs::data_dir().unwrap();
    regs_asl_path.push("mra_parser");
    regs_asl_path.push("regs.asl");

    match File::open(&regs_asl_path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", regs_asl_path.display(), e),
    }
}

fn main() {
    let mut file = init_state();
    let mut input = String::new();

    file.read_to_string(&mut input).unwrap();

    let data = parse_registers(&input);

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
