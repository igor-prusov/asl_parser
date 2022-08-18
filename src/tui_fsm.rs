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

enum TState<T, U> {
    Empty,
    Ambiguous(U),
    Selected(T),
}

trait Single<'a> {
    fn new(reg: &'a RegisterDesc, value: Option<u64>) -> Self;
    fn get_reg(&self) -> &'a RegisterDesc;
}

trait Multiple<'a> {
    fn new(vec: Vec<&'a RegisterDesc>, prefix: &str) -> Self;
    fn get_vec(&self) -> &Vec<&'a RegisterDesc>;
    fn get_prefix(&self) -> &str;
}

struct Fsm<'a, T: Single<'a>, U: Multiple<'a>> {
    data: FsmData<'a>,
    state: TState<T, U>,
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

impl<'a> Single<'a> for RegisterInfo<'a> {
    fn new(reg: &'a RegisterDesc, value: Option<u64>) -> Self {
        RegisterInfo {
            reg: reg,
            value: value,
        }
    }

    fn get_reg(&self) -> &'a RegisterDesc {
        self.reg
    }
}

impl<'a> Multiple<'a> for RegisterSubset<'a> {
    fn new(vec: Vec<&'a RegisterDesc>, prefix: &str) -> Self {
        RegisterSubset {
            vec,
            prefix: String::from(prefix),
        }
    }

    fn get_prefix(&self) -> &str {
        self.prefix.as_ref()
    }

    fn get_vec(&self) -> &Vec<&'a RegisterDesc> {
        &self.vec
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

impl<'a, 'b, T: Single<'b>, U: Multiple<'b>> TState<T, U> {
    fn from_prefix(prefix: &str, data: &'a FsmData<'b>) -> TState<T, U> {
        let m = data.select(prefix);
        match m.len() {
            0 => TState::Empty,
            1 => TState::Selected(T::new(m[0], None)),
            _ => TState::Ambiguous(U::new(m, prefix)),
        }
    }
}

impl<'a, T: Single<'a>, U: Multiple<'a> + Clone> Fsm<'a, T, U> {
    fn new(data: &'a BTreeMap<String, RegisterDesc>) -> Fsm<'a, T, U> {
        let d = FsmData::new(data);
        Fsm {
            data: d,
            state: TState::Empty,
        }
    }
    fn next(&mut self, event: Event) {
        self.state = match (&self.state, event) {
            /* From Empty */
            (TState::Empty, Event::Number(_)) => TState::Empty,
            (TState::Empty, Event::Text(s)) => TState::from_prefix(&s, &self.data),

            /* From Ambiguous */
            (TState::Ambiguous(subset), event) => match event {
                Event::Number(x) if (x as usize) < subset.get_vec().len() => {
                    TState::Selected(T::new(subset.get_vec()[x as usize], None))
                }
                _ => TState::Ambiguous(subset.clone()),
            },

            /* From Selected */
            (TState::Selected(reg), Event::Number(x)) => {
                TState::Selected(T::new(&reg.get_reg(), Some(x)))
            }

            (TState::Selected(_), Event::Text(value)) => TState::from_prefix(&value, &self.data),
        };
    }

    fn current(&'a self) -> &'a TState<T, U> {
        &self.state
    }

    fn prompt(&self) -> &str {
        match &self.state {
            TState::Empty => "",
            TState::Ambiguous(subset) => subset.get_prefix().as_ref(),
            TState::Selected(reg) => reg.get_reg().name.as_ref(),
        }
    }
}

struct FsmData<'a> {
    data: &'a BTreeMap<String, RegisterDesc>,
}

impl<'a> FsmData<'a> {
    fn new(data: &'a BTreeMap<String, RegisterDesc>) -> FsmData<'a> {
        FsmData { data }
    }
    fn select(&self, prefix: &str) -> Vec<&'a RegisterDesc> {
        let it = self
            .data
            .range(String::from(prefix)..)
            .take_while(|x| x.0.starts_with(&prefix));
        it.map(|(_, v)| v).collect()
    }
}

pub fn run_tui(data: &BTreeMap<String, RegisterDesc>) -> io::Result<()> {
    let mut fsm = Fsm::<RegisterInfo, RegisterSubset>::new(data);
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
