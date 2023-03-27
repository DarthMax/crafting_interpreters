use miette::SourceSpan;

#[derive(Debug, PartialEq, Clone)]
pub struct Position {
    pub absolute: usize,
    pub length: usize,
}

impl Position {
    pub(crate) fn new(absolute: usize, length: usize) -> Position {
        Position { absolute, length }
    }

    pub fn end_position(&self) -> usize {
        self.absolute + self.length
    }

    pub fn union(&mut self, other: &Position) {
        let start = self.absolute.min(other.absolute);
        let end = self.end_position().max(other.end_position());
        self.absolute = start;
        self.length = end - start;
    }
}

impl From<Position> for SourceSpan {
    fn from(val: Position) -> Self {
        (val.absolute, val.length).into()
    }
}
