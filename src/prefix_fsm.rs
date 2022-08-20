pub enum Event {
    Text(String),
    Number(u64),
}

pub enum TState<T> {
    Empty,
    Ambiguous(String, Vec<T>),
    Selected(T),
}

pub trait Item {
    fn update(&self, x: u64);
}

pub struct Fsm<T, F>
where
    F: Fn(&str) -> Vec<T>,
{
    pub state: TState<T>,
    pub from_prefix: F,
}

impl<T: Clone + Item, F: Fn(&str) -> Vec<T>> Fsm<T, F> {
    fn vec_to_state(prefix: &str, v: Vec<T>) -> TState<T> {
        match v.len() {
            0 => TState::Empty,
            1 => TState::Selected(v[0].clone()),
            _ => TState::Ambiguous(String::from(prefix), v),
        }
    }

    pub fn next(&mut self, event: Event) {
        self.state = match (&self.state, event) {
            /* From Empty */
            (TState::Empty, Event::Number(_)) => TState::Empty,
            (TState::Empty, Event::Text(s)) => {
                let v = (self.from_prefix)(&s);
                Self::vec_to_state(s.as_str(), v)
            }

            /* From Ambiguous */
            (TState::Ambiguous(prefix, vec), event) => match event {
                Event::Number(x) if (x as usize) < vec.len() => {
                    TState::Selected(vec[x as usize].clone())
                }
                _ => TState::Ambiguous(prefix.clone(), vec.clone()),
            },

            /* From Selected */
            (TState::Selected(reg), Event::Number(x)) => {
                reg.update(x);
                TState::Selected(reg.clone())
            }

            (TState::Selected(_), Event::Text(s)) => {
                let v = (self.from_prefix)(&s);
                Self::vec_to_state(s.as_str(), v)
            }
        }
    }

    pub fn new(f: F) -> Self {
        Fsm {
            state: TState::Empty,
            from_prefix: f,
        }
    }
}
