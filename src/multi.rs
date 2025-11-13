use std::collections::{Bound, HashMap};
use std::ops::{ControlFlow, RangeBounds};
use macros::{impl_tuples, swizzle_parsers};
use super::{StringReader, Parser, ParserOut, ParserError, Any, Repeatable, Branch, Permutation};

impl <F: Fn(StringReader) -> ParserOut<O>, O> Parser<O> for F {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<O> { self }
}

impl Parser<()> for () {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<()> { |input| Ok((input, ())) }
}

impl Any<()> for () {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<()> { |input| Ok((input, ())) }
}

impl <F: Fn(StringReader) -> ParserOut<O>, O> Any<O> for F {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<O> { self }
}

impl <P: Parser<O>, O> Branch<O> for P {
    fn branch<O2>(self, skip_match: bool, if_ok: impl Parser<O2>, if_error: impl Parser<O2>) -> impl Fn(StringReader) -> ParserOut<O2> {
        let parser = self.parser();
        let if_ok = if_ok.parser();
        let if_error = if_error.parser();
        move |input| {
            if let Ok((reader, _)) = parser(input.clone()) {
                if_ok(if skip_match { reader } else { input })
            } else {
                if_error(input)
            }
        }
    }
}

impl_tuples!(26);

impl <P: Parser<O>, O> Parser<Vec<O>> for Vec<P> {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
        let mut parsers = Vec::with_capacity(self.len());
        for p in self {
            parsers.push(p.parser());
        }
        move |mut input| {
            let mut o;
            let mut out = Vec::with_capacity(parsers.len());
            for t in &parsers {
                (input, o) = t(input)?;
                out.push(o);
            }
            Ok((input, out))
        }
    }
}

impl <P: Parser<O>, O> Any<O> for Vec<P> {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<O> {
        let mut parsers = Vec::with_capacity(self.len());
        for p in self {
            parsers.push(p.parser());
        }
        move |input| {
            for p in &parsers {
                if let Ok((reader, o)) = p(input.clone()) {
                    return Ok((reader, o));
                }
            }
            Err(ParserError::NoMatch { head: input.true_index(0) })
        }
    }
}

impl <P: Parser<O>, O: Clone> Permutation<Vec<O>> for Vec<P> {
    fn permute(self) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
        let mut parsers = Vec::with_capacity(self.len());
        for p in self {
            parsers.push(p.parser());
        }
        move |mut input| {
            let mut tmap = HashMap::with_capacity(parsers.len());
            for _ in 0..parsers.len() {
                let mut found = false;
                for i in 0..parsers.len() {
                    if tmap.contains_key(&i) { continue; }
                    if let Ok((reader, o)) = parsers[i](input.clone()) {
                        tmap.insert(i, o);
                        input = reader;
                        found = true;
                        break;
                    }
                }
                if found == false {
                    return Err(ParserError::NoMatch { head: input.true_index(0) });
                }
            }
            let mut out = Vec::new();
            for i in 0..parsers.len() {
                out.push((*tmap.get(&i).unwrap()).clone());
            }
            Ok((input, out))
        }
    }
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.parse` and `Parser`
pub fn seq<P: Parser<O>, O>(sequence: P) -> impl Fn(StringReader) -> ParserOut<O> {
    sequence.parser()
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.any` and `Any`
pub fn any<A: Any<O>, O>(any: A) -> impl Fn(StringReader) -> ParserOut<O> {
    any.any()
}

pub fn perm<P: Permutation<O>, O>(permutable: P) -> impl Fn(StringReader) -> ParserOut<O> {
    permutable.permute()
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.branch` and `Branch`
pub fn branch<B: Parser<O1>, O1, O2>(skip_match: bool, branch: B, if_ok: impl Parser<O2>, if_error: impl Parser<O2>) -> impl Fn(StringReader) -> ParserOut<O2> {
    branch.branch(skip_match, if_ok, if_error)
}

impl <F: Parser<O>, O> Repeatable<O> for F {
    fn rep<R: RangeBounds<usize>>(self, range: R, greedy: bool) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
        let min = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0
        };
        let max = match range.end_bound() {
            Bound::Included(v) => Some(*v),
            Bound::Excluded(v) => Some(*v - 1),
            Bound::Unbounded => None
        };
        let parser = self.parser();
        move |mut input| {
            let mut out = Vec::with_capacity(min);
            while greedy || max.map_or(true, |max| out.len() < max) {
                if let Ok((ni, o)) = parser(input.clone()) {
                    out.push(o);
                    input = ni;
                } else {
                    break;
                }
            }
            if range.contains(&out.len()) {
                Ok((input, out))
            } else {
                Err(ParserError::MatchedOutsideOfRange { matched: out.len(), min, max })
            }
        }
    }

    fn rep_separated<_D, R: RangeBounds<usize>>(self, separator: impl Parser<_D>, range: R, greedy: bool) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
        let min = match range.start_bound() {
            Bound::Included(v) => *v,
            Bound::Excluded(v) => *v + 1,
            Bound::Unbounded => 0
        };
        let max = match range.end_bound() {
            Bound::Included(v) => Some(*v),
            Bound::Excluded(v) => Some(*v - 1),
            Bound::Unbounded => None
        };
        let parser = self.parser();
        let separator = separator.parser();
        move |mut input| {
            let mut out = Vec::with_capacity(min);
            while greedy || max.map_or(true, |max| out.len() < max) {
                if let Ok((ni, o)) = parser(input.clone()) {
                    out.push(o);
                    input = ni;
                } else {
                    break;
                }
                if let Ok((ni, _)) = separator(input.clone()) {
                    input = ni;
                } else {
                    break;
                }
            }
            if range.contains(&out.len()) {
                Ok((input, out))
            } else {
                Err(ParserError::MatchedOutsideOfRange { matched: out.len(), min, max })
            }
        }
    }
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.rep` and 'Repeatable`
pub fn rep<R: RangeBounds<usize>, F: Repeatable<O>, O>(range: R, greedy: bool, rep: F) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
    rep.rep(range, greedy)
}

///helper function to make the parser expression more readable (puts the action at the start of the expression instead of the end)
///see `.rep_separated` and 'Repeatable`
pub fn rep_separated<R: RangeBounds<usize>, F: Repeatable<O>, O, _D>(range: R, greedy: bool, rep: F, separator: impl Parser<_D>) -> impl Fn(StringReader) -> ParserOut<Vec<O>> {
    rep.rep_separated(separator, range, greedy)
}

///parse 3 expressions, but only keep the result of the middle one (discarding the first and last results)
pub fn delimited<_D1, F: Parser<O>, O, _D2>(before: impl Parser<_D1>, target: F, after: impl Parser<_D2>) -> impl Fn(StringReader) -> ParserOut<O> {
    swizzle_parsers!((before, target, after); 1)
}

///parse 2 expressions and discard the first result (keeping the second result)
pub fn preceded<_D, F: Parser<O>, O>(before: impl Parser<_D>, target: F) -> impl Fn(StringReader) -> ParserOut<O> {
    swizzle_parsers!((before, target); 1)
}

///parse 2 expressions and keep the first result (discarding the second result)
pub fn terminated<F: Parser<O>, O, _D>(target: F, after: impl Parser<_D>) -> impl Fn(StringReader) -> ParserOut<O> {
    swizzle_parsers!((target, after); 0)
}

///parse 3 expressions and keep only the first and last result, discarding the middle one
pub fn separated_pair<F1: Parser<O1>, O1, _D, F2: Parser<O2>, O2>(first: F1, separator: impl Parser<_D>, second: F2) -> impl Fn(StringReader) -> ParserOut<(O1, O2)> {
    swizzle_parsers!((first, separator, second); 0, 2)
}

///folds repeatedly the input using control flows to tell when to stop folding (with an error or a valid resul)
pub fn fold<O: Clone>(start: O, consumer: impl Fn(O, StringReader)->ControlFlow<ParserOut<O>, (StringReader, O)>)->impl Fn(StringReader) -> ParserOut<O> {
    move |mut input| {
        let mut acc = start.clone();
        loop {
            match consumer(acc, input) {
                ControlFlow::Continue((i, o)) => {
                    input = i;
                    acc = o;
                }
                ControlFlow::Break(o) => { return  o; }
            }
        }
    }
}

pub fn take_while(predicate: impl Fn(char) -> bool) -> impl Fn(StringReader) -> ParserOut<String> {
    fold(String::new(), move |mut acc, input| {
        let c = input[0];

        if c == '\0' || !predicate(c) { return ControlFlow::Break(Ok((input, acc))); }

        acc.push(c);
        match input.move_head(1) {
            Ok(next) => ControlFlow::Continue((next, acc)),
            Err(err) => ControlFlow::Break(Err(err)),
        }
    })
}

#[cfg(test)]
mod test {
    use crate::multi::take_while;
    use crate::Parseable;

    #[test]
    fn take_while_ident() {
        assert_eq!("Id: Jhon".parse_with(false, take_while(|c| c.is_alphabetic())).unwrap(), "Id".to_string());
    }
}