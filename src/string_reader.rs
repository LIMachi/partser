use std::ops::Index;
use std::rc::Rc;
use super::{Parseable, Parser, ParserError, StringReader, StringView};

impl Index<usize> for StringReader {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        if self.head + index < self.chars.len() {
            &self.chars[self.head + index]
        } else {
            &'\0'
        }
    }
}

impl StringReader {
    pub fn move_head_mut(&mut self, amount: isize) -> Result<(), ParserError> {
        if amount > 0 {
            self.head += amount as usize;
            if self.head > self.chars.len() {
                Err(ParserError::EndOfInput)
            } else {
                Ok(())
            }
        } else if amount < 0 {
            let d = -amount as usize;
            if d > self.head {
                Err(ParserError::EndOfInput)
            } else {
                self.head -= d;
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    pub fn move_head(&self, amount: isize) -> Result<Self, ParserError> {
        if amount > 0 {
            let mut n = self.clone();
            n.head += amount as usize;
            if n.head > n.chars.len() {
                Err(ParserError::EndOfInput)
            } else {
                Ok(n)
            }
        } else if amount < 0 {
            let mut n = self.clone();
            let d = -amount as usize;
            if d > n.head {
                Err(ParserError::EndOfInput)
            } else {
                n.head -= d;
                Ok(n)
            }
        } else {
            Ok(self.clone())
        }
    }

    pub fn skip_whitespaces(self) -> Self {
        let mut i = self.head;
        while i < self.chars.len() && self.chars[i].is_whitespace() {
            i += 1;
        }
        if i != self.head {
            Self {
                string: self.string.clone(),
                chars: self.chars.clone(),
                head: i
            }
        } else {
            self
        }
    }

    pub fn finished(self) -> Result<(), ParserError> {
        if self.head >= self.chars.len() {
            Ok(())
        } else {
            let mut acc = String::new();
            for i in self.head..self.chars.len() {
                acc.push(self.chars[i]);
            }
            Err(ParserError::DanglingCharacters { head: self.head, length: self.chars.len(), left_to_parse: acc })
        }
    }

    pub fn true_index(&self, index: usize) -> usize { self.head + index }

    pub fn view(&self, len: isize) -> StringView {
        if self.head >= self.chars.len() {
            StringView::empty(self.string.clone())
        } else if len >= 0 {
            StringView::new(self.string.clone(), self.head..self.head + len as usize)
        } else {
            StringView::new(self.string.clone(), self.head - (-len as usize)..self.head)
        }
    }
}

impl Into<StringReader> for &str {
    fn into(self) -> StringReader {
        StringReader {
            string: Rc::<str>::from(self),
            chars: Rc::new(self.chars().collect()),
            head: 0,
        }
    }
}

impl Into<StringReader> for String {
    fn into(self) -> StringReader {
        let chars = Rc::new(self.chars().collect());
        StringReader {
            string: Rc::<str>::from(self),
            chars,
            head: 0,
        }
    }
}

impl <O, S: Into<StringReader>> Parseable<O> for S {
    fn parse_with(self, all: bool, parser: impl Parser<O>) -> Result<O, ParserError> {
        parser.parser()(self.into()).and_then(|(reader, o)| {
            if all {
                reader.finished()?;
            }
            Ok(o)
        })
    }
}