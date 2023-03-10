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
}

impl From<Position> for SourceSpan {
    fn from(val: Position) -> Self {
        (val.absolute, val.length).into()
    }
}
