use std::{env::args, fs::File, io::Read};

use mra_parser::parse_registers;

mod asl_helpers;
use asl_helpers::{build_regs_asl, regs_asl_path};
use tui_fsm::run_tui;

mod tui_fsm;

fn init_state() -> File {
    let path = regs_asl_path();

    match File::open(&path) {
        Ok(x) => x,
        Err(e) => panic!("Can't open {}: {}", path.display(), e),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = args().collect();
    if args.len() > 1 && args[1] == "init" {
        if let Err(e) = build_regs_asl().await {
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
