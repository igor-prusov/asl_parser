pub enum Event {
    Text(String),
    Number(u64),
}

pub enum TState {
    Empty,
    Ambiguous(Vec<String>),
    Selected(String),
}

pub trait Single {}
pub trait Multiple {
    fn len(&self) -> usize;
    fn at(&self, index: usize) -> dyn Single;
}

pub struct Fsm<F, G>
where
    F: Fn(&str) -> TState,
    G: Fn(&str) -> String,
{
    pub state: TState,
    pub from_prefix: F,
    pub print_one: G,
}

impl<F: Fn(&str) -> TState, G: Fn(&str) -> String> Fsm<F, G> {
    pub fn next(&mut self, event: Event) {
        self.state = match (&self.state, event) {
            /* From Empty */
            (TState::Empty, Event::Number(_)) => TState::Empty,
            (TState::Empty, Event::Text(s)) => (self.from_prefix)(&s),

            /* From Ambiguous */
            (TState::Ambiguous(vec), event) => match event {
                Event::Number(x) if (x as usize) < vec.len() => {
                    TState::Selected(vec[x as usize].clone())
                }
                _ => TState::Ambiguous(vec.clone()),
            },

            /* From Selected */
            (TState::Selected(reg), Event::Number(x)) => TState::Selected(reg.clone()),

            (TState::Selected(_), Event::Text(prefix)) => (self.from_prefix)(&prefix),
        }
    }

    /*
    pub fn prompt(&self) -> String {
        match &self.state {
            TState::Empty => String::from(""),
            TState::Ambiguous(vec) => vec[0].clone(),
            TState::Selected(name) => (self.print_one)(name),
        }
    }
    */
}
