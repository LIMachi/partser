use std::ops::{Bound, RangeBounds};
use std::rc::Rc;
use crate::StringView;

impl StringView {
    pub fn empty(string: Rc<str>) -> Self {
        Self {
            string,
            start: Bound::Excluded(0),
            end: Bound::Excluded(0)
        }
    }

    pub fn new(string: Rc<str>, range: impl RangeBounds<usize>) -> Self {
        Self {
            string,
            start: range.start_bound().cloned(),
            end: range.end_bound().cloned(),
        }
    }

    pub fn as_str(&self) -> &str {
        if self.start == Bound::Excluded(0) && self.end == Bound::Excluded(0) {
            &""
        } else {
            &self.string[(self.start, self.end)]
        }
    }
}