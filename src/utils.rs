use crate::{Any, ExpectedChar, Parser, ParserError, ParserOut, StringReader, StringView};

impl <'s> Parser<&'s str> for &'s str {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<&'s str> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for (i, c) in chars.iter().enumerate() {
                if input[i] != *c {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(i), char: input[i], expected: ExpectedChar::Single(*c) });
                }
            }
            Ok((input.move_head(chars.len() as isize)?, self))
        }
    }
}

impl Parser<String> for String {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<String> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for (i, c) in chars.iter().enumerate() {
                if input[i] != *c {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(i), char: input[i], expected: ExpectedChar::Single(*c) });
                }
            }
            Ok((input.move_head(chars.len() as isize)?, self.clone()))
        }
    }
}

impl Any<char> for &str {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<char> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for c in &chars {
                if input[0] == *c {
                    return Ok((input.move_head(1)?, *c));
                }
            }
            Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::Any(self.to_string()) })
        }
    }
}

impl Any<char> for String {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<char> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for c in &chars {
                if input[0] == *c {
                    return Ok((input.move_head(1)?, *c));
                }
            }
            Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::Any(self.clone()) })
        }
    }
}

impl Parser<char> for char {
    fn parser(self) -> impl Fn(StringReader) -> ParserOut<char> {
        move |input| {
            if input[0] == self {
                Ok((input.move_head(1)?, self))
            } else {
                Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::Single(self) })
            }
        }
    }
}

impl Any<char> for char {
    fn any(self) -> impl Fn(StringReader) -> ParserOut<char> {
        move |input| {
            if input[0] == self {
                Ok((input.move_head(1)?, self))
            } else {
                Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::Single(self) })
            }
        }
    }
}

pub fn white(input: StringReader) -> ParserOut<usize> {
    let mut t = 0;
    while input[t].is_whitespace() {
        t += 1;
    }
    Ok((input.move_head(t as isize)?, t))
}

pub fn skip(chars: usize) -> impl Fn(StringReader) -> ParserOut<usize> {
    move |input| { Ok((input.move_head(chars as isize)?, chars)) }
}

pub fn single(input: StringReader) -> ParserOut<char> {
    let c = input[0];
    Ok((input.move_head(1)?, c))
}

pub trait NoneOf {
    fn none_of(self) -> impl Fn(StringReader) -> ParserOut<char>;
}

impl NoneOf for &str {
    fn none_of(self) -> impl Fn(StringReader) -> ParserOut<char> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for c in &chars {
                if input[0] == *c {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::NoneOf(self.to_string()) });
                }
            }
            let c = input[0];
            return Ok((input.move_head(1)?, c));
        }
    }
}

impl NoneOf for String {
    fn none_of(self) -> impl Fn(StringReader) -> ParserOut<char> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            for c in &chars {
                if input[0] == *c {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::NoneOf(self.clone()) });
                }
            }
            let c = input[0];
            return Ok((input.move_head(1)?, c));
        }
    }
}

pub trait CaseInsensitive<O>: Sized {
    fn case_insensitive(self) -> impl Fn(StringReader) -> ParserOut<O>;
}

impl CaseInsensitive<String> for String {
    fn case_insensitive(self) -> impl Fn(StringReader) -> ParserOut<Self> {
        let chars: Vec<char> = self.chars().collect();
        move |input| {
            let mut acc = Vec::with_capacity(chars.len());
            for (i, c) in chars.iter().enumerate() {
                if !input[i].eq_ignore_ascii_case(c) {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(i), char: input[i], expected: ExpectedChar::Single(*c) });
                }
                acc.push(*c);
            }
            Ok((input.move_head(chars.len() as isize)?, String::from_iter(acc)))
        }
    }
}

impl CaseInsensitive<StringView> for &str {
    fn case_insensitive(self) -> impl Fn(StringReader) -> ParserOut<StringView> {
        let chars: Vec<char> = self.chars().collect();
        move |mut input| {
            let mut len = 0;
            for (i, c) in chars.iter().enumerate() {
                if !input[i].eq_ignore_ascii_case(c) {
                    return Err(ParserError::InvalidCharacter { pos: input.true_index(i), char: input[i], expected: ExpectedChar::Single(*c) });
                }
                len += c.len_utf8() as isize;
            }
            input.move_head_mut(chars.len() as isize)?;
            let view = input.view(-len);
            Ok((input, view))
        }
    }
}

impl CaseInsensitive<char> for char {
    fn case_insensitive(self) -> impl Fn(StringReader) -> ParserOut<Self> {
        move |input| {
            if input[0].eq_ignore_ascii_case(&self) {
                Ok((input.move_head(1)?, input[0]))
            } else {
                Err(ParserError::InvalidCharacter { pos: input.true_index(0), char: input[0], expected: ExpectedChar::Single(self) })
            }
        }
    }
}

pub trait MapErrToString<O> {
    fn map_err_to_string(self) -> Result<O, String>;
}

impl <O> MapErrToString<O> for Result<O, ParserError> {
    fn map_err_to_string(self) -> Result<O, String> {
        self.map_err(|e| format!("{e:?}"))
    }
}