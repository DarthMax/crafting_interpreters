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

impl Into<SourceSpan> for Position {
    fn into(self) -> SourceSpan {
        (self.absolute, self.length).into()
    }
}
