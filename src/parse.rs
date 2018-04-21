#[cfg(debug_assertions)]
const _GRAMMAR: &'static str = include_str!("calc.pest");

#[derive(Parser)]
#[grammar = "calc.pest"]
pub struct CalcParser;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn integer() {
        parses_to! {
            parser: CalcParser,
            input: "143",
            rule: Rule::int,
            tokens: [
                int(0, 3)
            ]
        };
    }

    #[test]
    fn negative_integer() {
        parses_to! {
            parser: CalcParser,
            input: "-143",
            rule: Rule::int,
            tokens: [
                int(0, 4)
            ]
        };
    }

    #[test]
    fn float() {
        parses_to! {
            parser: CalcParser,
            input: "0.05",
            rule: Rule::float,
            tokens: [
                float(0, 4)
            ]
        };
    }

    #[test]
    fn negative_float() {
        parses_to! {
            parser: CalcParser,
            input: "-8829.5",
            rule: Rule::float,
            tokens: [
                float(0, 7)
            ]
        };
    }

    #[test]
    fn rational() {
        parses_to! {
            parser: CalcParser,
            input: "76/99",
            rule: Rule::rational,
            tokens: [
                rational(0, 5)
            ]
        };
    }

    #[test]
    fn negative_rational() {
        parses_to! {
            parser: CalcParser,
            input: "-1/9",
            rule: Rule::rational,
            tokens: [
                rational(0, 4)
            ]
        };
    }

    #[test]
    fn scientific() {
        parses_to! {
            parser: CalcParser,
            input: "50E2 - 0.3345E6",
            rule: Rule::expr,
            tokens: [
                expr(0, 15, [
                    int(0, 4),
                    sub(5, 6),
                    float(7, 15),
                ])
            ]
        };
    }

    #[test]
    fn pair() {
        parses_to! {
            parser: CalcParser,
            input: "(1 + 4)",
            rule: Rule::expr,
            tokens: [
                expr(0, 7, [
                    int(1, 2),
                    add(3, 4),
                    int(5, 6),
                ])
            ]
        };
    }
}
