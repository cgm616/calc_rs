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
        Operator::new(Rule::pow, Assoc::Right),
        Operator::new(Rule::rem, Assoc::Left),
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

#[derive(Clone, Copy, Debug)]
enum InfoType {
    About,
    Help,
}

#[derive(Clone, Debug)]
enum Object {
    Integer(i128),
    Float(f64),
    Error(String),
    Info(InfoType),
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

impl Rem for Object {
    type Output = Object;
    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs % rhs),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs % rhs),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float(lhs as f64 % rhs as f64),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs as f64 % rhs as f64),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }
}

impl Object {
    fn pow(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs.pow(rhs as u32)),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs.powf(rhs)),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float((lhs as f64).powf(rhs)),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs.powi(rhs as i32)),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }

    fn display(self) -> Option<HtmlElement> {
        // A macro to create `p` elements.
        macro_rules! new_text_node {
            ($text:expr) => {{
                let display: HtmlElement = document().create_element("p").unwrap().try_into().unwrap();
                display.append_child(&document().create_text_node($text));
                display
            }};
        }

        // A macro to create links with specified text and location.
        macro_rules! new_link_node {
            ($href:expr, $text:expr) => {{
                let link: HtmlElement = document().create_element("a").unwrap().try_into().unwrap();
                link.append_child(&document().create_text_node($text));
                link.set_attribute("href", $href);
                link
            }};
        }

        match self {
            Object::Integer(int) => Some(new_text_node!(&int.to_string())),
            Object::Float(float) => Some(new_text_node!(&float.to_string())),
            Object::Error(string) => {
                let display = new_text_node!(&string);
                display.class_list().add("error").unwrap();
                Some(display)
            }
            Object::Info(InfoType::About) => {
                let container: HtmlElement = document()
                    .create_element("div")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let display1 = new_text_node!(
                    "This is a REPL calculator running on the web. It was made with the "
                );
                display1.append_child(&new_link_node!(
                    "https://www.rust-lang.org",
                    "Rust programming language"
                ));
                display1.append_child(&document().create_text_node(" and "));
                display1.append_child(&new_link_node!("https://github.com/koute/stdweb", "stdweb"));
                display1.append_child(&document().create_text_node(", a library for constructing client side web apps in Rust. It was compiled to "));
                display1.append_child(&new_link_node!(
                    "https://developer.mozilla.org/en-US/docs/WebAssembly",
                    "WebAssembly"
                ));
                display1.append_child(&document().create_text_node(" and then included in an html file to run on the web. To see the source code, check it out on "));
                display1.append_child(&new_link_node!(
                    "https://github.com/cgm616/calc_rs",
                    "Github."
                ));
                let line_break: HtmlElement =
                    document().create_element("br").unwrap().try_into().unwrap();
                let display2 = new_text_node!("Try running `help()` for more info.");

                container.append_child(&display1);
                container.append_child(&line_break);
                container.append_child(&display2);
                container.class_list().add("info").unwrap();
                Some(container)
            }
            Object::Info(InfoType::Help) => {
                let container: HtmlElement = document()
                    .create_element("div")
                    .unwrap()
                    .try_into()
                    .unwrap();
                let display1 = new_text_node!("Use any of the following operations:");
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1.append_child(&document().create_text_node("+ for addition"));
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1.append_child(&document().create_text_node("- for subtraction"));
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1.append_child(&document().create_text_node("* for multiplication"));
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1.append_child(&document().create_text_node("/ for division"));
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1.append_child(&document().create_text_node("^ for exponentation"));
                display1.append_child::<HtmlElement>(&document()
                    .create_element("br")
                    .unwrap()
                    .try_into()
                    .unwrap());
                display1
                    .append_child(&document().create_text_node("= for assignment of variables (ex: `a = b`)"));
                let line_break1: HtmlElement =
                    document().create_element("br").unwrap().try_into().unwrap();
                let display2 = new_text_node!("Try using a few well known constants, like `pi` and `e`. `ans` is a special variable that is always the last result.");
                let line_break2: HtmlElement =
                    document().create_element("br").unwrap().try_into().unwrap();
                let display3 = new_text_node!("Be careful with order of operations. It doesn't quite work yet, so use parentheses when in doubt. Also, negative numbers are not supported! (yet)");

                container.append_child(&display1);
                container.append_child(&line_break1);
                container.append_child(&display2);
                container.append_child(&line_break2);
                container.append_child(&display3);
                container.class_list().add("info").unwrap();
                Some(container)
            }
            Object::Nil => None,
        }
    }

    fn set_ans(&self, state: &StateRef) {
        match self {
            &Object::Integer(int) => {
                state
                    .borrow_mut()
                    .assignments
                    .insert("ans".to_string(), int.into());
            }
            &Object::Float(float) => {
                state
                    .borrow_mut()
                    .assignments
                    .insert("ans".to_string(), float.into());
            }
            _ => {}
        };
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

    let first_line: HtmlElement = document()
        .query_selector("#latest")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();

    let first_prompt: HtmlElement = first_line.last_child().unwrap().try_into().unwrap();

    first_prompt.set_text_content("about()");

    let result = eval(&state, "about()");
    show(&state, result);
    new_prompt(&state);
}

fn add_input_events(state: &StateRef, element: &HtmlElement) {
    element.add_event_listener(enclose!( (element, state) move |event: KeyPressEvent| {
        if event.key() == "Enter" {
            event.prevent_default();

            let entry: String = element.inner_text();

            if !entry.is_whitespace() {
                state.borrow_mut().add_entry(&entry);
                let result = eval(&state, &entry);
                result.set_ans(&state);
                show(&state, result);
                new_prompt(&state);
            } else {
                new_prompt(&state);
            }
        } else if event.key() == "ArrowUp" {
            event.prevent_default();

            match state.borrow_mut().next_history() {
                Some(string) => element.set_text_content(string),
                None => {}
            }
        } else if event.key() == "ArrowDown" {
            event.prevent_default();

            match state.borrow_mut().previous_history() {
                Some(string) => element.set_text_content(string),
                None => element.set_text_content("")
            }
        }
    }));

    element.add_event_listener(enclose!( (element, state) move |event: InputEvent| {
        let incomplete: String = element.inner_text();
            if !incomplete.is_whitespace() {
                let result = eval(&state, &incomplete);
                match result {
                    Object::Error(_text) => element.class_list().add("error").unwrap(),
                    _ => element.class_list().remove("error").unwrap()
                };
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
                    Rule::add => lhs.add(rhs),
                    Rule::sub => lhs.sub(rhs),
                    Rule::mul => lhs.mul(rhs),
                    Rule::div => lhs.div(rhs),
                    Rule::rem => lhs.rem(rhs),
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
            Rule::help => Object::Info(InfoType::Help),
            Rule::about => Object::Info(InfoType::About),
            _ => unreachable!(),
        }
    }

    match pairs.clone().next() {
        Some(pair) => consume(state, pair),
        None => Object::Error("".to_string()),
    }
}

fn show(state: &StateRef, output: Object) {
    // Ask the output to construct a DOM to display itself, and then see if it
    // gives one.
    match output.display() {
        Some(html) => {
            // If it does, find the console then add the DOM.
            let console: HtmlElement = document()
                .query_selector("#console")
                .unwrap()
                .unwrap()
                .try_into()
                .unwrap();

            // Construct the new line container and add the right class.
            let new_line: HtmlElement = document()
                .create_element("div")
                .unwrap()
                .try_into()
                .unwrap();
            new_line.class_list().add("line");

            // Add the html from the Object to the new line and add the line to
            // the console.
            new_line.append_child(&html);
            console.append_child(&new_line);
        }
        None => {} // If it doesn't, do nothing.
    }
}

fn new_prompt(state: &StateRef) {
    // Since this is a new prompt, reset the history counter.
    state.borrow_mut().reset_counter();

    // Find the element that is the previous prompt. Remove the special id for
    // the latest prompt.
    let previous_line: HtmlElement = document()
        .query_selector("#latest")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    previous_line.remove_attribute("id");

    // The second child of the previous prompt should be the input box. Make it
    // uneditable.
    let previous_input: HtmlElement = previous_line.last_child().unwrap().try_into().unwrap();
    previous_input
        .set_attribute("contenteditable", "false")
        .unwrap();

    // Construct a new prompt div and give it the special id.
    let new_line: HtmlElement = document()
        .create_element("div")
        .unwrap()
        .try_into()
        .unwrap();
    new_line.set_attribute("id", "latest").unwrap();
    new_line.class_list().add("line").unwrap();

    // Construct the actual prompt text that comes before the input. This is a
    // pre element with a text node.
    let new_prompt: HtmlElement = document()
        .create_element("pre")
        .unwrap()
        .try_into()
        .unwrap();
    new_prompt.class_list().add("prompt").unwrap();
    new_prompt.append_child(&document().create_text_node("calc_rs > "));

    // Construct the contenteditable p element that works as an input.
    let new_input: HtmlElement = document().create_element("p").unwrap().try_into().unwrap();
    new_input.set_attribute("contenteditable", "true").unwrap();
    new_input.class_list().add("input").unwrap();

    // Add the callbacks on the events to the new input.
    add_input_events(state, &new_input);

    // Add the prompt text and the input to the line container.
    new_line.append_child(&new_prompt);
    new_line.append_child(&new_input);

    // Find the list of lines and add the new line to the list.
    let console = document().query_selector("#console").unwrap().unwrap();
    console.append_child(&new_line);

    // Focus on the new input.
    new_input.focus();
}
