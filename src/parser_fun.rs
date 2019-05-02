#[derive(Clone, Debug, PartialEq, Eq)]
struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

type ParseResult<'a, Output> = Result<(&'a str, Output), &'a str>;

trait Parser<'a, Output> {
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output>;
}

impl<'a, F, Output> Parser<'a, Output> for F
where
    F: Fn(&'a str) -> ParseResult<Output>,
{
    fn parse(&self, input: &'a str) -> ParseResult<'a, Output> {
        self(input)
    }
}

fn match_letter(c: char, input: &str) -> ParseResult<()> {
    match input.chars().next() {
        Some(letter) if letter == c => Ok((&input[letter.len_utf8()..], ())),
        _ => Err(input),
    }
}

fn match_literal<'a>(expected: &'a str) -> impl Parser<'a, &str> {
    move |input: &'a str| match input.get(0..expected.len()) {
        Some(next) if next == expected => Ok((&input[expected.len()..], expected)),
        _ => Err(input),
    }
}

fn match_ident(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    };

    while let Some(next) = chars.next() {
        if next.is_alphabetic() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

fn pair<'a, P1, P2, R1, R2>(p1: P1, p2: P2) -> impl Parser<'a, (R1, R2)>
where
    P1: Parser<'a, R1>,
    P2: Parser<'a, R2>,
{
    move |input| {
        p1.parse(input).and_then(|(new_input, res1)| {
            p2.parse(new_input)
                .map(|(rest_input, res2)| (rest_input, (res1, res2)))
        })
    }
}

fn map<'a, P, F, A, B>(parser: P, map_fn: F) -> impl Parser<'a, B>
where
    P: Parser<'a, A>,
    F: Fn(A) -> B,
{
    move |input| {
        parser
            .parse(input)
            .map(|(next, result)| (next, map_fn(result)))
    }
}

fn zero_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |input| {
        let mut result = Vec::new();
        let mut to_parse = input;
        while let Ok((rest, parsed)) = parser.parse(to_parse) {
            result.push(parsed);
            to_parse = rest;
        }
        Ok((to_parse, result))
    }
}

fn one_or_more<'a, P, A>(parser: P) -> impl Parser<'a, Vec<A>>
where
    P: Parser<'a, A>,
{
    move |input| {
        let mut result = Vec::new();
        let mut to_parse = input;
        if let Ok((rest, parsed)) = parser.parse(to_parse) {
            result.push(parsed);
            to_parse = rest;
        } else {
            return Err(input);
        }
        while let Ok((rest, parsed)) = parser.parse(to_parse) {
            result.push(parsed);
            to_parse = rest;
        }
        Ok((to_parse, result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter() {
        let c = '\u{1F601}';
        let phrase = "\u{1F601} smile";
        assert_eq!(match_letter(c, phrase), Ok((" smile", ())));
    }

    #[test]
    fn test_literal() {
        let lit = "\u{1F601}";
        let phrase = "\u{1F601} smile";
        assert_eq!(match_literal(lit).parse(phrase), Ok((" smile", lit)));
    }

    #[test]
    fn test_ident() {
        let phrase = "demo-id>";
        assert_eq!(match_ident(phrase), Ok((">", String::from("demo-id"))));
    }

    #[test]
    fn test_pair() {
        let phrase = "<demo-id>";
        let less_parser = match_literal("<");
        let great_parser = match_literal(">");
        let pair_parser = pair(pair(less_parser, match_ident), great_parser);
        assert_eq!(
            pair_parser.parse(phrase),
            Ok(("", (("<", String::from("demo-id")), ">")))
        );
    }

    #[test]
    fn test_zero_or_more() {
        #[derive(Debug, PartialEq, Eq)]
        struct Ident {
            val: String,
        };

        let phrase = "<demo-id><kaspa><xxx>";
        let less_parser = match_literal("<");
        let great_parser = match_literal(">");
        let ident_parser = map(match_ident, |id| Ident { val: id });
        let pair_parser = pair(pair(less_parser, ident_parser), great_parser);
        assert_eq!(
            zero_or_more(pair_parser).parse(phrase),
            Ok((
                "",
                vec![
                    (
                        (
                            "<",
                            Ident {
                                val: String::from("demo-id")
                            }
                        ),
                        ">"
                    ),
                    (
                        (
                            "<",
                            Ident {
                                val: String::from("kaspa")
                            }
                        ),
                        ">"
                    ),
                    (
                        (
                            "<",
                            Ident {
                                val: String::from("xxx")
                            }
                        ),
                        ">"
                    )
                ]
            ))
        )
    }
}
