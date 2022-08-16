use std::{
    collections::BTreeMap,
    fmt,
    io::{self, Write},
};

use mra_parser::RegisterDesc;

enum Event {
    Text(String),
    Number(u64),
}

struct RegisterInfo<'a> {
    reg: &'a RegisterDesc,
    value: Option<u64>,
}

#[derive(Clone)]
struct RegisterSubset<'a> {
    vec: Vec<&'a RegisterDesc>,
    prefix: String,
}

enum TState<'a> {
    Empty,
    Ambiguous(RegisterSubset<'a>),
    Selected(RegisterInfo<'a>),
}

struct Fsm<'a> {
    data: &'a BTreeMap<String, RegisterDesc>,
    state: TState<'a>,
}

impl<'a> fmt::Display for RegisterInfo<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.reg)?;
        if let Some(x) = self.value {
            writeln!(f, "value = {}", x)?;
        };
        Ok(())
    }
}

impl<'a> fmt::Display for RegisterSubset<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, reg) in self.vec.iter().enumerate() {
            writeln!(f, "{}: {}", i, reg.name)?;
        }
        Ok(())
    }
}

impl<'a> RegisterInfo<'a> {
    fn new(desc: &'a RegisterDesc) -> Self {
        RegisterInfo {
            reg: desc,
            value: None,
        }
    }
}

impl<'a> RegisterSubset<'a> {
    fn new(vec: Vec<&'a RegisterDesc>, prefix: &str) -> RegisterSubset<'a> {
        RegisterSubset {
            vec,
            prefix: String::from(prefix),
        }
    }
}

impl<'a> TState<'a> {
    fn from_prefix(prefix: &str, data: &'a BTreeMap<String, RegisterDesc>) -> TState<'a> {
        let it = data
            .range(String::from(prefix)..)
            .take_while(|x| x.0.starts_with(&prefix));
        let m: Vec<&RegisterDesc> = it.map(|(_, v)| v).collect();
        match m.len() {
            0 => TState::Empty,
            1 => TState::Selected(RegisterInfo::new(m[0])),
            _ => TState::Ambiguous(RegisterSubset::new(m, prefix)),
        }
    }
}

impl<'a> Fsm<'a> {
    fn new(data: &'a BTreeMap<String, RegisterDesc>) -> Fsm<'a> {
        Fsm {
            data,
            state: TState::Empty,
        }
    }
    fn next(&mut self, event: Event) {
        self.state = match (&self.state, event) {
            /* From Empty */
            (TState::Empty, Event::Number(_)) => TState::Empty,
            (TState::Empty, Event::Text(s)) => TState::from_prefix(&s, self.data),

            /* From Ambiguous */
            (TState::Ambiguous(subset), event) => match event {
                Event::Number(x) if (x as usize) < subset.vec.len() => {
                    TState::Selected(RegisterInfo::new(subset.vec[x as usize]))
                }
                _ => TState::Ambiguous(subset.clone()),
            },

            /* From Selected */
            (TState::Selected(reg), Event::Number(x)) => TState::Selected(RegisterInfo {
                reg: reg.reg,
                value: Some(x),
            }),

            (TState::Selected(_), Event::Text(value)) => TState::from_prefix(&value, self.data),
        };
    }

    fn current(&'a self) -> &'a TState<'a> {
        &self.state
    }

    fn prompt(&self) -> &str {
        match &self.state {
            TState::Empty => "",
            TState::Ambiguous(subset) => subset.prefix.as_ref(),
            TState::Selected(reg) => reg.reg.name.as_ref(),
        }
    }
}

pub fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> io::Result<()> {
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
        match fsm.current() {
            TState::Selected(x) => println!("{}", x),
            TState::Ambiguous(x) => println!("{}", x),
            TState::Empty => (),
        }
    }
    Ok(())
}
