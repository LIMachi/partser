/*
use std::collections::HashSet;
use std::ops::ControlFlow;
use crate::{Any, Parseable, Parser, ParserError, ParserOut, Repeatable, StringReader};
use crate::mappers::{take_fold, Mappable};
use crate::multi::{branch, delimited, any, terminated};
use crate::prelude::separated_pair;
use crate::utils::single;

pub trait Regex {
    fn regex(self) -> impl Fn(StringReader) -> ParserOut<Vec<String>>;
}

fn char_or_range() -> impl Fn(StringReader) -> ParserOut<HashSet<char>> {
    any((
        separated_pair(single, '-', single).map_ok(|(min, max)| {
            let mut out = HashSet::new();
            for c in min..=max {
                out.insert(c);
            }
            out
        }),
        take_fold(HashSet::new(), |mut state, char, input| {
            if (char == '-' || char == ']') && input[1] != ']' {
                ControlFlow::Break(if state.is_empty() { Err(ParserError::EndOfInput) } else { Ok((input, state)) })
            } else {
                state.insert(char);
                ControlFlow::Continue(state)
            }
        }),
        terminated("-]".any(), ']').map_ok(|c| {
            let mut out = HashSet::new();
            out.insert(c);
            out
        })
    ))
}

//transforms [a], [^a-c], [-]] and others to a parser module that matches a single character in or out (if set starts with ^) of the set
fn brackets(input: StringReader) -> ParserOut<impl Fn(StringReader) -> ParserOut<char>> {
    fn vec_hashset_char_to_string(input: Vec<HashSet<char>>) -> String {
        let mut all = HashSet::new();
        for h in &input {
            all.extend(h);
        }
        let mut out = String::new();
        for c in &all {
            out.push(*c);
        }
        out
    }
    delimited('[', branch(true, '^',
                          char_or_range().rep(1.., false).map_ok(|v| (true, v)),
                          char_or_range().rep(1.., false).map_ok(|v| (false, v))
    ), ']').map_ok(|(neg, arr)| {
        let any = vec_hashset_char_to_string(arr).any();
        move |input: StringReader| {
            if neg {
                if any(input.clone()).is_err() {
                    let c = input[0];
                    Ok((input.move_head(1)?, c))
                } else {
                    Err(ParserError::NoMatch { head: input.true_index(0) })
                }
            } else {
                any(input)
            }
        }
    }).parser()(input)
}

#[test]
fn test_brackets() {
    let br = "[a-c-]]".parse_with(false, brackets).unwrap();
    dbg!("abcac--]afaaab".parse_with(false, br.rep(.., true)));
}

impl <P: Parseable<Vec<String>>> Regex for P {
    fn regex(self) -> impl Fn(StringReader) -> ParserOut<Vec<String>> {
        move |input| {
            Ok((input, vec![]))
        }
    }
}*/