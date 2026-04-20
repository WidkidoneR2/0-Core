//! 50k line ring buffer scrollback
use crate::terminal::Cell;
pub struct Scrollback {
    lines:    Vec<Vec<Cell>>,
    max_lines: usize,
}
impl Scrollback {
    pub fn new(max_lines: usize) -> Self {
        Self { lines: Vec::new(), max_lines }
    }
    pub fn push(&mut self, line: Vec<Cell>) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
        }
        self.lines.push(line);
    }
    pub fn len(&self) -> usize { self.lines.len() }
    pub fn get(&self, idx: usize) -> Option<&Vec<Cell>> { self.lines.get(idx) }
}
