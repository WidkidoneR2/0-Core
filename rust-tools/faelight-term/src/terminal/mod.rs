//! faelight-term v2 -- Terminal State (Phase 0 stub)
//! VTE parser, cell grid, 50k line scrollback.
pub mod grid;
pub mod scrollback;
use vte::{Params, Parser, Perform};
/// Terminal grid cell
#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub ch: char,
    pub fg: u8,
    pub bg: u8,
    pub attrs: CellAttrs,
}

/// Cell attributes
#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttrs {
    pub bold:          bool,
    pub italic:        bool,
    pub dim:           bool,
    pub strikethrough: bool,
    pub underline:     bool,
    pub blink:         bool,
    pub reverse:       bool,
}
/// Terminal state
pub struct Terminal {
    pub cols:      usize,
    pub rows:      usize,
    pub grid:      Vec<Vec<Cell>>,
    pub cursor_x:  usize,
    pub cursor_y:  usize,
    pub scrollback: scrollback::Scrollback,
    parser:        Parser,
}
impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            grid: vec![vec![Cell::default(); cols]; rows],
            cursor_x: 0,
            cursor_y: 0,
            scrollback: scrollback::Scrollback::new(50_000),
            parser: Parser::new(),
        }
    }
    /// Feed bytes from PTY into the VTE parser
    pub fn feed(&mut self, data: &[u8]) {
        let mut handler = VteHandler { term: self };
        for &byte in data {
            let mut p = Parser::new();
            p.advance(&mut handler, byte);
        }
    }
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.grid = vec![vec![Cell::default(); cols]; rows];
    }
}
struct VteHandler<'a> {
    term: &'a mut Terminal,
}
impl<'a> Perform for VteHandler<'a> {
    fn print(&mut self, c: char) {
        if self.term.cursor_x < self.term.cols && self.term.cursor_y < self.term.rows {
            self.term.grid[self.term.cursor_y][self.term.cursor_x].ch = c;
            self.term.cursor_x += 1;
            if self.term.cursor_x >= self.term.cols {
                self.term.cursor_x = 0;
                self.term.cursor_y += 1;
                if self.term.cursor_y >= self.term.rows {
                    self.term.cursor_y = self.term.rows - 1;
                    // scroll up
                    let row = self.term.grid.remove(0);
                    self.term.scrollback.push(row);
                    self.term.grid.push(vec![Cell::default(); self.term.cols]);
                }
            }
        }
    }
    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.term.cursor_y += 1;
                if self.term.cursor_y >= self.term.rows {
                    self.term.cursor_y = self.term.rows - 1;
                    let row = self.term.grid.remove(0);
                    self.term.scrollback.push(row);
                    self.term.grid.push(vec![Cell::default(); self.term.cols]);
                }
            }
            b'\r' => { self.term.cursor_x = 0; }
            b'\x08' => { if self.term.cursor_x > 0 { self.term.cursor_x -= 1; } }
            _ => {}
        }
    }
    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn csi_dispatch(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
}
