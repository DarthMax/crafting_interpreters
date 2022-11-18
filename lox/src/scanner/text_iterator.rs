use std::vec::IntoIter;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Entry {
    pub(crate) value: char,
    pub(crate) position: u32,
    pub(crate) column: u32,
    pub(crate) line: u32,
}

impl Entry {
    fn new(value: char, position: u32, column: u32, line: u32) -> Entry {
        Entry { value, position, column, line }
    }
}

pub(crate) struct SourceIterator {
    source: String,
    chars: IntoIter<char>,
    peek: Option<Option<char>>,
    peek_next: Option<Option<char>>,
    pos: u32,
    line: u32,
    column: u32,
}

impl Iterator for SourceIterator {
    type Item = Entry;

    fn next(&mut self) -> Option<Self::Item> {
        let next_value = match self.peek.take() {
            Some(c) => {
                self.peek = self.peek_next.take();
                c
            }
            None => {
                match self.peek_next.take() {
                    Some(c) => c,
                    None => self.chars.next()
                }
            }
        };

        match next_value {
            Some(e) => {
                let position = self.pos;
                let line = self.line;
                let column = self.column;

                match e {
                    '\n' => {
                        self.pos += 1;
                        self.column = 0;
                        self.line += 1;
                    }
                    _ => {
                        self.pos += 1;
                        self.column += 1;
                    }
                };
                Some(Entry::new(e, position , column, line))
            }
            None => None
        }
    }
}

impl SourceIterator {
    pub(crate) fn new(text: String) -> SourceIterator {
        let chars = text.chars().collect::<Vec<_>>().into_iter();

        SourceIterator {
            source: text,
            chars,
            peek: None,
            peek_next: None,
            pos: 0,
            column: 0,
            line: 0,
        }
    }

    pub(crate) fn peek(&mut self) -> Option<char> {
        let chars = &mut self.chars;
        *self.peek.get_or_insert_with(|| chars.next())
    }

    pub(crate) fn peek_next(&mut self) -> Option<char> {
        self.peek();

        let chars = &mut self.chars;
        *self.peek_next.get_or_insert_with(|| {
            chars.next()
        })
    }

    pub(crate) fn next_match(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.next();
                true
            }
            _ => false
        }
    }

    pub(crate) fn scan_until(&mut self, target: char) -> Option<Entry> {
        loop {
            match self.next() {
                Some(c) if c.value == target => return Some(c),
                None => return None,
                Some(_) => ()
            }
        }
    }

    pub(crate) fn substring(&self, from: u32, to: u32) -> String {
        let text: &str = &self.source;
        text[from as usize..=to as usize].to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::SourceIterator;
    use super::*;

    #[test]
    fn test_next_should_return_available_elements() {
        let mut iterator = SourceIterator::new("Foo b\na".to_string());

        assert_eq!(iterator.next(), Some(Entry::new('F', 0, 0, 0)));
        assert_eq!(iterator.next(), Some(Entry::new('o', 1, 1, 0)));
        assert_eq!(iterator.next(), Some(Entry::new('o', 2, 2, 0)));
        assert_eq!(iterator.next(), Some(Entry::new(' ', 3, 3, 0)));
        assert_eq!(iterator.next(), Some(Entry::new('b', 4, 4, 0)));
        assert_eq!(iterator.next(), Some(Entry::new('\n', 5, 5, 0)));
        assert_eq!(iterator.next(), Some(Entry::new('a', 6, 0, 1)));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_peek_looks_ahead_one() {
        let mut iterator = SourceIterator::new("Fo".to_string());

        assert_eq!(iterator.peek(), Some('F'));
        assert_eq!(iterator.next(), Some(Entry::new('F', 0, 0, 0)));
        assert_eq!(iterator.peek(), Some('o'));
        iterator.next();
        assert_eq!(iterator.peek, None);
    }

    #[test]
    fn test_peek_can_be_run_multiple_times() {
        let mut iterator = SourceIterator::new("Fo".to_string());

        assert_eq!(iterator.peek(), Some('F'));
        assert_eq!(iterator.peek(), Some('F'));
        assert_eq!(iterator.next(), Some(Entry::new('F', 0, 0, 0)));
    }

    #[test]
    fn test_peek_next_looks_ahead_two() {
        let mut iterator = SourceIterator::new("Bar".to_string());

        assert_eq!(iterator.peek_next(), Some('a'));
        assert_eq!(iterator.next(), Some(Entry::new('B', 0, 0, 0)));
        assert_eq!(iterator.peek_next(), Some('r'));
        iterator.next();
        assert_eq!(iterator.peek_next, None);
    }

    #[test]
    fn test_peek_next_can_be_run_multiple_times() {
        let mut iterator = SourceIterator::new("Bar".to_string());

        assert_eq!(iterator.peek_next(), Some('a'));
        assert_eq!(iterator.peek_next(), Some('a'));
        assert_eq!(iterator.next(), Some(Entry::new('B', 0, 0, 0)));
    }

    #[test]
    fn test_peek_and_peek_next_work_together() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());

        assert_eq!(iterator.peek(), Some('B'));
        assert_eq!(iterator.peek_next(), Some('a'));
        assert_eq!(iterator.next(), Some(Entry::new('B', 0, 0, 0)));
        assert_eq!(iterator.peek(), Some('a'));
        assert_eq!(iterator.peek_next(), Some('r'));
    }

    #[test]
    fn test_next_match_returns_true_if_next_character_matches() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        assert!(iterator.next_match('B'));
    }

    #[test]
    fn test_next_match_advances_the_iterator_on_match() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        iterator.next_match('B');
        assert_eq!(iterator.next(), Some(Entry::new('a', 1, 1, 0)));
    }

    #[test]
    fn test_next_match_returns_false_if_characters_do_not_match() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        assert!(!iterator.next_match('a'));
    }

    #[test]
    fn test_next_match_does_not_advance_on_no_match() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        iterator.next_match('a');
        assert_eq!(iterator.next(), Some(Entry::new('B', 0, 0, 0)));
    }

    #[test]
    fn test_scan_until_finds_first_match() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        assert_eq!(iterator.scan_until('a'), Some(Entry::new('a', 1, 1, 0)));
        assert_eq!(iterator.scan_until('a'), Some(Entry::new('a', 4, 4, 0)));
    }

    #[test]
    fn test_scan_until_consumes_iterator_on_no_match() {
        let mut iterator = SourceIterator::new("BarBaz".to_string());
        assert_eq!(iterator.scan_until('x'), None);
        assert_eq!(iterator.next(), None)
    }

    #[test]
    fn test_substring() {
        let iterator = SourceIterator::new("BarBaz".to_string());
        assert_eq!(iterator.substring(1,2), "ar");
        assert_eq!(iterator.substring(0,0), "B");
    }
}
