use std::{
    collections::BTreeMap,
    fmt,
    io::{self, Write},
};

use crate::prefix_fsm::{Event, Fsm, Item, TState};
use mra_parser::RegisterDesc;

#[derive(Clone)]
struct Elem<'a>(&'a RegisterDesc);

impl<'a> Item for Elem<'a> {
    fn update(&self, x: u64) {
        println!("value = {}", x)
    }
}

impl<'a> fmt::Display for Elem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn get_prompt<'a>(state: &'a TState<Elem>) -> &'a str {
    match &state {
        TState::Empty => "",
        TState::Ambiguous(prefix, _) => prefix,
        TState::Selected(reg) => reg.0.name.as_ref(),
    }
}

pub fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> io::Result<()> {
    let mut fsm = Fsm::new(|prefix: &str| -> Vec<Elem> {
        let it = data
            .range(String::from(prefix)..)
            .take_while(|x| x.0.starts_with(&prefix));

        it.map(|(_, v)| Elem(v)).collect()
    });

    println!("Enter register names:");
    loop {
        print!("{}> ", get_prompt(&fsm.state));
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
        if let TState::Selected(el) = &fsm.state {
            println!("{}", el.0);
        }
        if let TState::Ambiguous(_, v) = &fsm.state {
            for (i, x) in v.iter().enumerate() {
                println!("{}) {}", i, x.0.name)
            }
        }
    }
    Ok(())
}
