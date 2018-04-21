use std::ops::{Add, Div, Mul, Rem, Sub};
use stdweb::{traits::*,
             unstable::TryInto,
             web::{document, HtmlElement}};

#[derive(Clone, Copy, Debug)]
pub enum InfoType {
    About,
    Help,
}

#[derive(Clone, Debug)]
pub enum Object {
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
    pub fn pow(self, rhs: Self) -> Self {
        match (self, rhs) {
            (Object::Integer(lhs), Object::Integer(rhs)) => Object::Integer(lhs.pow(rhs as u32)),
            (Object::Float(lhs), Object::Float(rhs)) => Object::Float(lhs.powf(rhs)),
            (Object::Integer(lhs), Object::Float(rhs)) => Object::Float((lhs as f64).powf(rhs)),
            (Object::Float(lhs), Object::Integer(rhs)) => Object::Float(lhs.powi(rhs as i32)),
            _ => Object::Error("that operation isn't supported".to_string()),
        }
    }

    pub fn display(self) -> Option<HtmlElement> {
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
                link.set_attribute("href", $href).unwrap();
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
                let display3 = new_text_node!("Negative numbers are not yet supported!");

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
