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
    fn update(&mut self, x: u64);
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
                let mut r = reg.clone();
                r.update(x);
                TState::Selected(r)
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

#[cfg(test)]
mod tests {
    use crate::prefix_fsm::{Event, Fsm, Item, TState};
    #[derive(Clone)]
    struct Elem(String, Option<u64>);
    impl Elem {
        fn new(s: &str) -> Elem {
            Elem(String::from(s), None)
        }
    }
    impl Item for Elem {
        fn update(&mut self, x: u64) {
            self.1 = Some(x);
        }
    }

    fn make_text(s: &str) -> Event {
        Event::Text(String::from(s))
    }

    fn assert_selected(state: &TState<Elem>, s: &str) {
        assert!(matches!(state, TState::Selected(Elem(_, None))));
        if let TState::Selected(Elem(x, None)) = state {
            assert_eq!(x, s)
        }
    }

    fn assert_empty(state: &TState<Elem>) {
        assert!(matches!(state, TState::Empty));
    }

    #[test]
    fn test() {
        let fsm_create = || {
            Fsm::new(|prefix| -> Vec<Elem> {
                let data = vec!["Single", "Multiple1", "Multiple2"];
                let mut v = Vec::new();

                for x in data {
                    if x.starts_with(prefix) {
                        v.push(Elem::new(x));
                    }
                }

                v
            })
        };

        /* Empty */
        let mut fsm = fsm_create();
        assert!(matches!(fsm.state, TState::Empty));

        fsm.next(Event::Number(1));
        assert_empty(&fsm.state);

        fsm.next(make_text("None"));
        assert_empty(&fsm.state);

        /* Empty -> Selected */
        fsm.next(make_text("Si"));
        assert_selected(&fsm.state, "Single");

        fsm.next(make_text("Si"));
        assert_selected(&fsm.state, "Single");

        /* Selected -> Empty */
        fsm.next(make_text("non-existent"));
    }
}
