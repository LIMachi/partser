use std::ops::ControlFlow;
use super::{StringReader, Parser, ParserOut};

pub trait Mappable<O>: Parser<O> {
    ///map the result of the parser to a new output (does nothing if error)
    fn map_ok<M: Fn(O) -> O2, O2>(self, map: M) -> impl Fn(StringReader) -> ParserOut<O2>;
    ///discards the result of the parser and replaces it with a default value (does nothing if error)
    fn default<O2: Clone>(self, default: O2) -> impl Fn(StringReader) -> ParserOut<O2>;
    fn and_then<M: Fn(StringReader, O) -> ParserOut<O2>, O2>(self, map: M) -> impl Fn(StringReader) -> ParserOut<O2>;
}

pub trait Optional<O>: Parser<O> {
    ///always succeeds, on ok, wrap the return in Some, on error returns Ok((<original_input>, None))
    fn optional(self) -> impl Fn(StringReader) -> ParserOut<Option<O>>;
}

impl <O, F: Parser<O>> Mappable<O> for F {
    fn map_ok<M: Fn(O) -> O2, O2>(self, map: M) -> impl Fn(StringReader) -> ParserOut<O2> {
        let parser = self.parser();
        move |input| {
            parser(input).map(|(take, o)| (take, map(o)))
        }
    }

    fn default<O2: Clone>(self, default: O2) -> impl Fn(StringReader) -> ParserOut<O2> {
        let parser = self.parser();
        move |input| parser(input).map(|(reader, _)| (reader, default.clone()))
    }

    fn and_then<M: Fn(StringReader, O) -> ParserOut<O2>, O2>(self, map: M) -> impl Fn(StringReader) -> ParserOut<O2> {
        let parser = self.parser();
        move |input| {
            parser(input).and_then(|(reader, out)| map(reader, out))
        }
    }
}

impl <O, F: Parser<O>> Optional<O> for F {
    fn optional(self) -> impl Fn(StringReader) -> ParserOut<Option<O>> {
        let parser = self.parser();
        move |input| {
            if let Ok((reader, o)) = parser(input.clone()) {
                Ok((reader, Some(o)))
            } else {
                Ok((input, None))
            }
        }
    }
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.map` and `Mappable`
pub fn map<F: Parser<O1>, O1, M: Fn(O1) -> O2, O2>(parser: F, map: M) -> impl Fn(StringReader) -> ParserOut<O2> {
    parser.map_ok(map)
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.default` and `Mappable`
pub fn default<F: Parser<O1>, O1, O2: Clone>(parser: F, default: O2) -> impl Fn(StringReader) -> ParserOut<O2> {
    parser.default(default)
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.optional` and `Optional`
pub fn optional<F: Parser<O>, O>(parser: F) -> impl Fn(StringReader) -> ParserOut<Option<O>> {
    parser.optional()
}

///takes character while the fold function returns true, returning the state at the end (never fails)
pub fn take_fold<S: Clone, F: Fn(S, char, StringReader) -> ControlFlow<ParserOut<S>, S>>(start: S, fold: F) -> impl Fn(StringReader) -> ParserOut<S> {
    move |mut input| {
        let mut state = start.clone();
        loop {
            match fold(state, input[0], input.clone()) {
                ControlFlow::Continue(s) => {
                    if let Ok(t) = input.move_head(1) {
                        input = t;
                        state = s;
                    } else {
                        return Ok((input, s));
                    }
                }
                ControlFlow::Break(out) => {
                    return out;
                }
            }
        }
    }
}