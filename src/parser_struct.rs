#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

type ParseResult<Output> = Result<(String, Output), String>;

trait ParserStruct<Output> {
    fn parse(&self, input: String) -> ParseResult<Output>;
}

#[derive(Clone)]
struct LiteralParser {
    expected: String,
}

impl ParserStruct<String> for LiteralParser {
    fn parse(&self, input: String) -> ParseResult<String> {
        match input.get(0..self.expected.len()) {
            Some(next) if next == self.expected => {
                let rest = &input[self.expected.len()..];
                Ok((rest.to_string(), self.expected.clone()))
            }
            _ => Err(input),
        }
    }
}

#[derive(Clone)]
struct PairParser<'a, A, B> {
    parser_a: &'a ParserStruct<A>,
    parser_b: &'a ParserStruct<B>,
}

impl<'a, A, B> ParserStruct<(A, B)> for PairParser<'a, A, B> {
    fn parse(&self, input: String) -> ParseResult<(A, B)> {
        self.parser_a.parse(input).and_then(|(new_input, res1)| {
            self.parser_b
                .parse(new_input)
                .map(|(rest_input, res2)| (rest_input, (res1, res2)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let lit = String::from("\u{1F601}");
        let phrase = String::from("\u{1F601} smile");
        let parser = LiteralParser {
            expected: lit.clone(),
        };
        assert_eq!(parser.parse(phrase), Ok((String::from(" smile"), lit)));
    }

    #[test]
    fn test_pair() {
        let phrase = String::from("<>");
        let parser_less = LiteralParser {
            expected: String::from("<"),
        };
        let parser_great = LiteralParser {
            expected: String::from(">"),
        };
        let pair_parser = PairParser {
            parser_a: &parser_less,
            parser_b: &parser_great,
        };
        assert_eq!(
            pair_parser.parse(phrase),
            Ok((String::from(""), (String::from("<"), String::from(">"))))
        );
    }
}
