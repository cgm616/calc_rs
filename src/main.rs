#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate pest;
#[macro_use]
extern crate lazy_static;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::rc::Rc;

use stdweb::{traits::*,
             unstable::TryInto,
             web::{document,
                   event::{InputEvent, KeyPressEvent},
                   html_element::InputElement,
                   HtmlElement}};

use pest::{iterators::Pair,
           iterators::Pairs,
           prec_climber::{Assoc, Operator, PrecClimber},
           Error as PestError,
           Parser};

mod parse;
use parse::{CalcParser, Rule};

lazy_static! {
    static ref PREC_CLIMBER: PrecClimber<Rule> = PrecClimber::new(vec![
        Operator::new(Rule::sub, Assoc::Left) | Operator::new(Rule::add, Assoc::Left),
        Operator::new(Rule::mul, Assoc::Left) | Operator::new(Rule::div, Assoc::Left),
        Operator::new(Rule::pow, Assoc::Left),
    ]);
}

// Shamelessly stolen from stdweb's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

#[derive(Clone, Debug)]
enum Object {
    Integer(i128),
    Float(f64),
    Error(String),
    Info(String),
    Nil,
}

impl Add for Object {
    type Output = Object;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs + rhs),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs + rhs),
            (Object::Integer(lhs), Object::Float(rhs))
            | (Object::Float(rhs), Object::Integer(lhs)) => Object::Float(lhs as f64 + rhs as f64),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl Sub for Object {
    type Output = Object;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs - rhs),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs - rhs),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 - rhs as f64),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs as f64 - rhs as f64),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl Mul for Object {
    type Output = Object;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs * rhs),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs * rhs),
            (Object::Integer(lhs), Object::Float(rhs))
            | (Object::Float(rhs), Object::Integer(lhs)) => Object::Float(lhs as f64 * rhs as f64),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl Div for Object {
    type Output = Object;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs / rhs),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs / rhs),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 / rhs as f64),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs as f64 / rhs as f64),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl Object {
    fn is_error(&self) -> bool {
        match self {
            Object::Error(_) => true,
            _ => false,
        }
    }

    fn is_nil(&self) -> bool {
        match self {
            Object::Nil => true,
            _ => false,
        }
    }

    fn is_info(&self) -> bool {
        match self {
            Object::Info(_) => true,
            _ => false,
        }
    }

    fn pow(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs.pow(rhs as u32)),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs.powf(rhs)),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float((lhs as f64).powf(rhs)),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs.powi(rhs as i32)),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl From<i128> for Object {
    fn from(num: i128) -> Object {
        Object::Integer(num)
    }
}

impl From<f64> for Object {
    fn from(num: f64) -> Object {
        Object::Float(num)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            &Object::Integer(num) => num.fmt(fmt),
            &Object::Float(num) => num.fmt(fmt),
            &Object::Error(ref string) => string.fmt(fmt),
            &Object::Info(ref string) => string.fmt(fmt),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
struct State {
    history: Vec<String>,
    assignments: HashMap<String, Object>,
    counter: Option<usize>,
    last: Option<i128>,
}

type StateRef = Rc<RefCell<State>>;

impl State {
    fn new() -> Self {
        let mut map = HashMap::new();
        map.insert("pi".to_string(), Object::Float(std::f64::consts::PI));
        map.insert("e".to_string(), Object::Float(std::f64::consts::E));

        State {
            history: Vec::new(),
            assignments: map,
            counter: None,
            last: None,
        }
    }

    fn add_entry(&mut self, entry: &str) {
        let len = self.history.len();
        if (len > 0 && self.history[len - 1] != entry) || len == 0 {
            self.history.push(entry.to_string());
        }
    }

    fn counter_exists(&self) -> bool {
        self.counter.is_some()
    }

    fn next_history(&mut self) -> Option<&str> {
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

    fn previous_history(&mut self) -> Option<&str> {
        if self.history.len() > 0 {
            match self.counter {
                Some(num) => {
                    if num < self.history.len() - 1 {
                        self.counter = Some(num + 1);
                        Some(&self.history[num + 1])
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            None
        }
    }

    fn reset_counter(&mut self) {
        self.counter = None;
    }
}

fn main() {
    let state = Rc::new(RefCell::new(State::new()));

    let latest: InputElement = document()
        .query_selector("#latest")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    latest.set_raw_value("about()");

    let result = eval(&state, "about()");
    show(&state, result);
    new_prompt(&state, false);
}

fn add_input_events(state: &StateRef, element: &InputElement) {
    element.add_event_listener(enclose!( (element, state) move |event: KeyPressEvent| {
        if event.key() == "Enter" {
            event.prevent_default();

            let entry: String = element.raw_value();

            if !entry.is_whitespace() {
                state.borrow_mut().add_entry(&entry);
                let result = eval(&state, &entry);
                if result.is_nil() {
                    new_prompt(&state, false);
                } else {
                    let err = result.is_error();
                    show(&state, result);
                    new_prompt(&state, err);
                }
            } else {
                new_prompt(&state, false);
            }
        } else if event.key() == "ArrowUp" {
            event.prevent_default();

            match state.borrow_mut().next_history() {
                Some(string) => element.set_raw_value(string),
                None => {}
            }
        } else if event.key() == "ArrowDown" {
            event.prevent_default();

            match state.borrow_mut().previous_history() {
                Some(string) => element.set_raw_value(string),
                None => {}
            }
        }
    }));

    element.add_event_listener(enclose!( (element, state) move |event: InputEvent| {
        let incomplete: String = element.raw_value();
            if !incomplete.is_whitespace() {
                let result = eval(&state, &incomplete);
                if result.is_error() {
                    element.class_list().add("error").unwrap();
                } else if element.class_list().contains("error") {
                    element.class_list().remove("error").unwrap();
                }
            }
    }));
}

fn eval(state: &StateRef, input: &str) -> Object {
    // follows P E (M | D) (A | S)

    let pairs = match CalcParser::parse(Rule::statement, input) {
        Ok(pairs) => pairs,
        Err(error) => return Object::Error("parsing error".to_string()),
    };

    // TODO: implement rational and power of ten exponents, maybe even bignums,
    // fix negative numbers, testing, order of ops working, responsive design,
    // make help messages work better, comment code, add readme, separate parse
    // into a library, add desktop gui, etc.
    fn consume(state: &StateRef, pair: Pair<Rule>) -> Object {
        match pair.as_rule() {
            Rule::assn => {
                // In an assignment, there must be exactly 2 pairs: `a = b`,
                // where a is a symbol and b is some kind of expression.
                let mut inner = pair.into_inner();
                let left = inner.next(); // symbol
                let right = consume(state, inner.next().unwrap()); // expr

                state // Insert the assignment
                    .borrow_mut()
                    .assignments
                    .insert(left.unwrap().as_str().to_string(), right);
                Object::Nil // and return nil to the machine.
            }
            Rule::expr => {
                let primary = |pair| consume(state, pair);

                let infix = |lhs: Object, op: Pair<Rule>, rhs: Object| match op.as_rule() {
                    Rule::pow => lhs.pow(rhs),
                    Rule::mul => lhs.mul(rhs),
                    Rule::div => lhs.div(rhs),
                    Rule::add => lhs.add(rhs),
                    Rule::sub => lhs.sub(rhs),
                    _ => unreachable!(),
                };

                PREC_CLIMBER.climb(pair.into_inner(), primary, infix)
            }
            Rule::symbol => match state.borrow().assignments.get(pair.as_str()) {
                Some(obj) => obj.clone(),
                None => Object::Error(format!("no variable named {}", pair.as_str())),
            },
            Rule::int => pair.as_str().parse::<i128>().unwrap().into(),
            Rule::float => pair.as_str().parse::<f64>().unwrap().into(),
            Rule::rational => unimplemented!(),
            // For help messages, I want them to be 80 chars across at maximum
            // on bigger displays, but still resize to be good on smaller ones.
            Rule::help => Object::Info(
                "Use any of the following operations:\n
                    + for addition\n
                    - for subtraction\n
                    * for multiplication\n
                    / for division\n
                    ^ for exponentation\n
                ------\n
                Be careful with order of operations. It doesn't quite work yet, so use parentheses when in doubt. Also, negative numbers not supported! (yet)"
                    .to_string(),
            ),
            Rule::about => Object::Info(
                "This is a REPL calculator running on the web. It was made with \
                the Rust programming language [1] and stdweb [2], a library for \
                constructing client side web apps in Rust. It was compiled to \
                WebAssembly [3] and then included in an html file.\n
                ------\n
                To see the source code, check it out on github [4].\n
                ------\n
                Try running `help()` for some basic help.\n
                ------\n
                [1]: https://www.rust-lang.org\n
                [2]: https://github.com/koute/stdweb\n
                [3]: https://developer.mozilla.org/en-US/docs/WebAssembly\n
                [4]: https://github/cgm616/calc_rs"
                    .to_string(),
            ),
            _ => unreachable!(),
        }
    }

    match pairs.clone().next() {
        Some(pair) => consume(state, pair),
        None => Object::Error("".to_string()),
    }
}

fn show(state: &StateRef, output: Object) {
    let error = output.is_error();
    let info = output.is_info();
    let output = format!("{}", output);

    let lines = output.lines();

    let entries: HtmlElement = document()
        .query_selector("#console")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    console!(log, "about to make div");

    let div: HtmlElement = document()
        .create_element("div")
        .unwrap()
        .try_into()
        .unwrap();
    console!(log, "made div");
    div.class_list().add("entry").unwrap();
    console!(log, "added class");

    for line in lines {
        let text: HtmlElement = document().create_element("p").unwrap().try_into().unwrap();
        text.append_child(&document().create_text_node(line));
        if error {
            text.class_list().add("error").unwrap();
        } else if info {
            text.class_list().add("info").unwrap();
        }

        div.append_child(&text);
    }

    entries.append_child(&div);
}

fn new_prompt(state: &StateRef, error: bool) {
    state.borrow_mut().reset_counter();

    let past: InputElement = document()
        .query_selector("#latest")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    past.set_attribute("disabled", "true").unwrap();
    past.remove_attribute("id");
    if past.class_list().contains("error") && !error {
        past.class_list().remove("error").unwrap();
    }

    let div: HtmlElement = document()
        .create_element("div")
        .unwrap()
        .try_into()
        .unwrap();
    div.class_list().add("entry").unwrap();
    div.class_list().add("input").unwrap();

    let input: InputElement = document()
        .create_element("input")
        .unwrap()
        .try_into()
        .unwrap();
    input.set_attribute("id", "latest").unwrap();

    add_input_events(&state, &input);

    let list = document().query_selector("#console").unwrap().unwrap();

    div.append_child(&input);
    list.append_child(&div);

    input.focus();
}
