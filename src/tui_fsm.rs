use std::{
    collections::BTreeMap,
    fmt,
    io::{self, Write},
};

use crate::prefix_fsm::{Event, Fsm, Item, TState};
use asl_parser::RegisterDesc;

#[derive(Clone)]
struct Elem<'a>(&'a RegisterDesc, Option<u64>);

impl<'a> Item for Elem<'a> {
    fn update(&mut self, x: u64) {
        self.1 = Some(x)
    }
}
impl<'a> fmt::Display for Elem<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            RegisterDesc {
                name: self.0.name.clone(),
                bits: self.0.bits,
                fields: self.0.fields.clone(),
                value: self.1,
            }
        )?;
        Ok(())
    }
}

fn get_prompt<'a>(state: &'a TState<Elem>) -> &'a str {
    match &state {
        TState::Empty => "",
        TState::Ambiguous(prefix, _) => prefix,
        TState::Selected(reg) => reg.0.name.as_ref(),
        TState::Final => "",
    }
}

pub fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> io::Result<()> {
    let mut fsm = Fsm::new(|prefix: &str| -> Vec<Elem> {
        data.range(String::from(prefix)..)
            .take_while(|x| x.0.starts_with(&prefix))
            .map(|p| Elem(p.1, None))
            .collect()
    });

    println!("Enter register names:");
    loop {
        print!("{}> ", get_prompt(&fsm.state));
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let event = Event::from_str(&input);

        fsm.next(event);

        match &&fsm.state {
            TState::Selected(el) => println!("{}", el),
            TState::Ambiguous(_, v) => {
                for (i, x) in v.iter().enumerate() {
                    println!("{}) {}", i, x.0.name)
                }
            }
            TState::Empty => (),
            TState::Final => break,
        }
    }
    Ok(())
}
