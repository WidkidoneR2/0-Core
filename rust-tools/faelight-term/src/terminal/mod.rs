//! faelight-term v2 -- Terminal State
pub mod grid;
pub mod scrollback;
use vte::{Params, Parser, Perform};
#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub ch:   char,
    pub fg:   u8,
    pub bg:   u8,
    pub attrs: CellAttrs,
}
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
pub struct Terminal {
    pub cols:       usize,
    pub rows:       usize,
    pub grid:       Vec<Vec<Cell>>,
    pub cursor_x:   usize,
    pub cursor_y:   usize,
    pub scrollback: scrollback::Scrollback,
    pub cur_fg:     u8,
    pub cur_bg:     u8,
    parser:         Parser,
}
impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            grid:       vec![vec![Cell::default(); cols]; rows],
            cursor_x:   0,
            cursor_y:   0,
            scrollback: scrollback::Scrollback::new(50_000),
            cur_fg:     7,
            cur_bg:     0,
            parser:     Parser::new(),
        }
    }
    pub fn feed(&mut self, data: &[u8]) {
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        {
            let mut handler = VteHandler { term: self };
            for &byte in data {
                parser.advance(&mut handler, byte);
            }
        }
        self.parser = parser;
    }
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.grid = vec![vec![Cell::default(); cols]; rows];
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
    }
    fn scroll_up(&mut self) {
        let row = self.grid.remove(0);
        self.scrollback.push(row);
        self.grid.push(vec![Cell::default(); self.cols]);
    }
    fn newline(&mut self) {
        self.cursor_y += 1;
        if self.cursor_y >= self.rows {
            self.cursor_y = self.rows - 1;
            self.scroll_up();
        }
    }
    fn put_char(&mut self, ch: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.newline();
        }
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.grid[self.cursor_y][self.cursor_x] = Cell {
                ch,
                fg: self.cur_fg,
                bg: self.cur_bg,
                attrs: CellAttrs::default(),
            };
            self.cursor_x += 1;
        }
    }
}
struct VteHandler<'a> {
    term: &'a mut Terminal,
}
impl<'a> Perform for VteHandler<'a> {
    fn print(&mut self, c: char) {
        self.term.put_char(c);
    }
    fn execute(&mut self, byte: u8) {
        match byte {
            0x0a => self.term.newline(),              // LF
            0x0d => self.term.cursor_x = 0,           // CR
            0x08 => {                                  // BS
                if self.term.cursor_x > 0 {
                    self.term.cursor_x -= 1;
                }
            }
            0x07 => {}  // BEL -- ignore
            0x1b => {}  // ESC -- handled by vte
            _    => {}
        }
    }
    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let ps: Vec<u16> = params.iter()
            .map(|p| p.first().copied().unwrap_or(0))
            .collect();
        let p0 = ps.get(0).copied().unwrap_or(0) as usize;
        let p1 = ps.get(1).copied().unwrap_or(0) as usize;
        match action {
            // Cursor movement
            'A' => { // CUU -- cursor up
                let n = p0.max(1);
                self.term.cursor_y = self.term.cursor_y.saturating_sub(n);
            }
            'B' => { // CUD -- cursor down
                let n = p0.max(1);
                self.term.cursor_y = (self.term.cursor_y + n).min(self.term.rows - 1);
            }
            'C' => { // CUF -- cursor forward
                let n = p0.max(1);
                self.term.cursor_x = (self.term.cursor_x + n).min(self.term.cols - 1);
            }
            'D' => { // CUB -- cursor back
                let n = p0.max(1);
                self.term.cursor_x = self.term.cursor_x.saturating_sub(n);
            }
            'H' | 'f' => { // CUP -- cursor position
                let row = if p0 == 0 { 0 } else { p0 - 1 };
                let col = if p1 == 0 { 0 } else { p1 - 1 };
                self.term.cursor_y = row.min(self.term.rows - 1);
                self.term.cursor_x = col.min(self.term.cols - 1);
            }
            'J' => { // ED -- erase display
                match p0 {
                    0 => { // clear from cursor to end
                        for x in self.term.cursor_x..self.term.cols {
                            self.term.grid[self.term.cursor_y][x] = Cell::default();
                        }
                        for y in (self.term.cursor_y + 1)..self.term.rows {
                            self.term.grid[y] = vec![Cell::default(); self.term.cols];
                        }
                    }
                    1 => { // clear from start to cursor
                        for y in 0..self.term.cursor_y {
                            self.term.grid[y] = vec![Cell::default(); self.term.cols];
                        }
                        for x in 0..=self.term.cursor_x {
                            self.term.grid[self.term.cursor_y][x] = Cell::default();
                        }
                    }
                    2 | 3 => { // clear all
                        self.term.grid = vec![vec![Cell::default(); self.term.cols]; self.term.rows];
                        self.term.cursor_x = 0;
                        self.term.cursor_y = 0;
                    }
                    _ => {}
                }
            }
            'K' => { // EL -- erase line
                match p0 {
                    0 => { // clear from cursor to end of line
                        for x in self.term.cursor_x..self.term.cols {
                            self.term.grid[self.term.cursor_y][x] = Cell::default();
                        }
                    }
                    1 => { // clear from start to cursor
                        for x in 0..=self.term.cursor_x {
                            self.term.grid[self.term.cursor_y][x] = Cell::default();
                        }
                    }
                    2 => { // clear entire line
                        self.term.grid[self.term.cursor_y] = vec![Cell::default(); self.term.cols];
                    }
                    _ => {}
                }
            }
            'm' => { // SGR -- select graphic rendition (colors)
                if ps.is_empty() || (ps.len() == 1 && ps[0] == 0) {
                    self.term.cur_fg = 7;
                    self.term.cur_bg = 0;
                    return;
                }
                let mut i = 0;
                while i < ps.len() {
                    match ps[i] {
                        0  => { self.term.cur_fg = 7; self.term.cur_bg = 0; }
                        30..=37 => { self.term.cur_fg = (ps[i] - 30) as u8; }
                        38 => { // 256 or truecolor fg -- skip for now
                            if ps.get(i+1).copied() == Some(5) { i += 2; }
                            else if ps.get(i+1).copied() == Some(2) { i += 4; }
                        }
                        39 => { self.term.cur_fg = 7; }
                        40..=47 => { self.term.cur_bg = (ps[i] - 40) as u8; }
                        48 => { // 256 or truecolor bg -- skip
                            if ps.get(i+1).copied() == Some(5) { i += 2; }
                            else if ps.get(i+1).copied() == Some(2) { i += 4; }
                        }
                        49 => { self.term.cur_bg = 0; }
                        90..=97  => { self.term.cur_fg = (ps[i] - 90 + 8) as u8; }
                        100..=107 => { self.term.cur_bg = (ps[i] - 100 + 8) as u8; }
                        _ => {}
                    }
                    i += 1;
                }
            }
            'l' | 'h' => {} // private modes -- ignore for now
            'r' => {} // DECSTBM -- scroll region, ignore for now
            _ => {}
        }
    }
    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'M' => { // RI -- reverse index (scroll up)
                if self.term.cursor_y > 0 {
                    self.term.cursor_y -= 1;
                }
            }
            _ => {}
        }
    }
}
