use std::{self, cell::RefCell, collections::HashMap, rc::Rc};

use super::Object;

#[derive(Clone)]
pub struct State {
    pub history: Vec<String>,
    pub assignments: HashMap<String, Object>,
    counter: Option<usize>,
}

pub type StateRef = Rc<RefCell<State>>;

impl State {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("pi".to_string(), Object::Float(std::f64::consts::PI));
        map.insert("π".to_string(), Object::Float(std::f64::consts::PI));
        map.insert("e".to_string(), Object::Float(std::f64::consts::E));

        State {
            history: Vec::new(),
            assignments: map,
            counter: None,
        }
    }

    pub fn add_entry(&mut self, entry: &str) {
        let len = self.history.len();
        if (len > 0 && self.history[len - 1] != entry) || len == 0 {
            self.history.push(entry.to_string());
        }
    }

    pub fn next_history(&mut self) -> Option<&str> {
        if self.history.len() > 0 {
            match self.counter {
                Some(num) => {
                    if num > 0 {
                        self.counter = Some(num - 1);
                        Some(&self.history[num - 1])
                    } else {
                        None
                    }
                }
                None => {
                    let num = self.history.len() - 1;
                    self.counter = Some(num);
                    Some(&self.history[num])
                }
            }
        } else {
            None
        }
    }

    pub fn previous_history(&mut self) -> Option<&str> {
        if self.history.len() > 0 {
            match self.counter {
                Some(num) => {
                    if num < self.history.len() - 1 {
                        self.counter = Some(num + 1);
                        Some(&self.history[num + 1])
                    } else {
                        self.counter = None;
                        None
                    }
                }
                None => None,
            }
        } else {
            None
        }
    }

    pub fn reset_counter(&mut self) {
        self.counter = None;
    }

    pub fn set_ans(&mut self, object: &Object) {
        match object {
            &Object::Integer(int) => {
                self.assignments.insert("ans".to_string(), int.into());
            }
            &Object::Float(float) => {
                self.assignments.insert("ans".to_string(), float.into());
            }
            _ => {}
        };
    }
}
