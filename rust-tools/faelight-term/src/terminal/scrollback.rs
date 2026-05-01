//! 50k line ring buffer scrollback
use crate::terminal::Cell;
pub struct Scrollback {
    lines: Vec<Vec<Cell>>,
    soft_wrapped: Vec<bool>,
    max_lines: usize,
}
impl Scrollback {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            soft_wrapped: Vec::new(),
            max_lines,
        }
    }
    pub fn push(&mut self, line: Vec<Cell>, soft_wrapped: bool) {
        if self.lines.len() >= self.max_lines {
            self.lines.remove(0);
            self.soft_wrapped.remove(0);
        }
        self.lines.push(line);
        self.soft_wrapped.push(soft_wrapped);
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    pub fn get(&self, idx: usize) -> Option<&Vec<Cell>> {
        self.lines.get(idx)
    }
    pub fn is_soft_wrapped(&self, idx: usize) -> bool {
        self.soft_wrapped.get(idx).copied().unwrap_or(false)
    }
}
