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

use pest::{iterators::Pair,
           prec_climber::{Assoc, Operator, PrecClimber},
           Parser};
use std::{cell::RefCell,
          ops::{Add, Div, Mul, Rem, Sub},
          rc::Rc};
use stdweb::{traits::*,
             unstable::TryInto,
             web::{document,
                   event::{InputEvent, KeyPressEvent},
                   HtmlElement}};

mod model;
use model::{InfoType, Object, State, StateRef};
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
                state.borrow_mut().set_ans(&result);
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

    element.add_event_listener(enclose!( (element, state) move |_event: InputEvent| {
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
        Err(_error) => return Object::Error("parsing error".to_string()),
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

fn show(_state: &StateRef, output: Object) {
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
            new_line.class_list().add("line").unwrap();

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
