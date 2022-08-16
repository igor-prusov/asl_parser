use std::{
    collections::BTreeMap,
    env::args,
    fs::File,
    io::{self, Read, Write},
};

use mra_parser::{parse_registers, RegisterDesc};

mod asl_helpers;
use asl_helpers::Result;
use asl_helpers::{prepare, regs_asl_path};
use tui_fsm::{Event, Fsm};

mod tui_fsm;

fn init_state() -> File {
    let path = regs_asl_path();

    match File::open(&path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", path.display(), e),
    }
}

fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> Result<()> {
    let mut fsm = Fsm::new(data);
    println!("Enter register names:");
    loop {
        print!("{}> ", fsm.prompt());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let input = input.trim().to_lowercase();
        if input.is_empty() {
            break;
        }

        let event = match input.parse::<u64>() {
            Ok(x) => Event::Number(x),
            Err(_) => Event::Text(input),
        };

        fsm.next(event);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "init" {
        if let Err(e) = prepare().await {
            panic!("Can't initialize regs.asl: {}", e);
        }
    }

    let mut file = init_state();
    let mut input = String::new();

    file.read_to_string(&mut input)
        .expect("Can't open regs.asl");

    let data = parse_registers(&input);

    run_tui(&data).expect("Error while interacting with user");
}
