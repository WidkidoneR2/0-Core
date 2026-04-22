//! faelight-term v2 -- Terminal State
pub mod grid;
pub mod scrollback;
use vte::{Params, Parser, Perform};
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }
    pub const DEFAULT_FG: Self = Self::rgb(0xd7, 0xe0, 0xda);
    pub const DEFAULT_BG: Self = Self::rgb(0x0f, 0x14, 0x11);
}
impl Default for Color {
    fn default() -> Self { Self::DEFAULT_FG }
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
#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub ch:    char,
    pub fg:    Color,
    pub bg:    Color,
    pub attrs: CellAttrs,
}
impl Default for Cell {
    fn default() -> Self {
        Self { ch: ' ', fg: Color::DEFAULT_FG, bg: Color::DEFAULT_BG, attrs: CellAttrs::default() }
    }
}
pub struct Terminal {
    pub cols:       usize,
    pub rows:       usize,
    pub grid:       Vec<Vec<Cell>>,
    pub cursor_x:   usize,
    pub cursor_y:   usize,
    pub scrollback: scrollback::Scrollback,
    pub cur_fg:     Color,
    pub cur_bg:     Color,
    cur_attrs:      CellAttrs,
    parser:         Parser,
}
impl Terminal {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols, rows,
            grid:       vec![vec![Cell::default(); cols]; rows],
            cursor_x:   0,
            cursor_y:   0,
            scrollback: scrollback::Scrollback::new(50_000),
            cur_fg:     Color::DEFAULT_FG,
            cur_bg:     Color::DEFAULT_BG,
            cur_attrs:  CellAttrs::default(),
            parser:     Parser::new(),
        }
    }
    pub fn feed(&mut self, data: &[u8]) {
        let mut parser = std::mem::replace(&mut self.parser, Parser::new());
        { let mut handler = VteHandler { term: self }; for &byte in data { parser.advance(&mut handler, byte); } }
        self.parser = parser;
    }
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols; self.rows = rows;
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
        if self.cursor_y >= self.rows { self.cursor_y = self.rows - 1; self.scroll_up(); }
    }
    fn put_char(&mut self, ch: char) {
        let cp = ch as u32;
        if matches!(cp, 0xFE00..=0xFE0F | 0x200B | 0x200C | 0x200D | 0x20D0..=0x20FF) { return; }
        if self.cursor_x >= self.cols { self.cursor_x = 0; self.newline(); }
        if self.cursor_y < self.rows && self.cursor_x < self.cols {
            self.grid[self.cursor_y][self.cursor_x] = Cell { ch, fg: self.cur_fg, bg: self.cur_bg, attrs: self.cur_attrs };
            let is_wide = matches!(cp, 0x1F000..=0x1FFFF);
            if is_wide && self.cursor_x + 1 < self.cols {
                self.cursor_x += 1;
                self.grid[self.cursor_y][self.cursor_x] = Cell { ch: '\0', fg: self.cur_fg, bg: self.cur_bg, attrs: self.cur_attrs };
                self.cursor_x += 1;
            } else { self.cursor_x += 1; }
        }
    }
}
fn color256(idx: u8) -> Color {
    const P: [(u8,u8,u8); 16] = [
        (0x0f,0x14,0x11),(0xe6,0x7e,0x80),(0x6b,0xe3,0xa3),(0xf5,0xc1,0x77),
        (0x5c,0xc8,0xff),(0xd6,0x99,0xb6),(0x7f,0xc8,0xc8),(0xd7,0xe0,0xda),
        (0x77,0x8f,0x7f),(0xe6,0x7e,0x80),(0x6b,0xe3,0xa3),(0xf5,0xc1,0x77),
        (0x5c,0xc8,0xff),(0xd6,0x99,0xb6),(0x7f,0xc8,0xc8),(0xff,0xff,0xff),
    ];
    if idx < 16 { let (r,g,b) = P[idx as usize]; return Color::rgb(r,g,b); }
    if idx >= 232 { let v = (8 + (idx-232) as u16 * 10).min(255) as u8; return Color::rgb(v,v,v); }
    let i = idx - 16;
    let b = i % 6; let g = (i/6)%6; let r = i/36;
    let f = |v: u8| if v==0 {0} else {55+v*40};
    Color::rgb(f(r),f(g),f(b))
}
struct VteHandler<'a> { term: &'a mut Terminal }
impl<'a> Perform for VteHandler<'a> {
    fn print(&mut self, c: char) { self.term.put_char(c); }
    fn execute(&mut self, byte: u8) {
        match byte {
            0x0a => self.term.newline(),
            0x0d => self.term.cursor_x = 0,
            0x08 => { if self.term.cursor_x > 0 { self.term.cursor_x -= 1; } }
            _ => {}
        }
    }
    fn csi_dispatch(&mut self, params: &Params, _: &[u8], _: bool, action: char) {
        let ps: Vec<u16> = params.iter().map(|p| p.first().copied().unwrap_or(0)).collect();
        let p0 = ps.first().copied().unwrap_or(0) as usize;
        let p1 = ps.get(1).copied().unwrap_or(0) as usize;
        match action {
            'A' => { let n=p0.max(1); self.term.cursor_y=self.term.cursor_y.saturating_sub(n); }
            'B' => { let n=p0.max(1); self.term.cursor_y=(self.term.cursor_y+n).min(self.term.rows-1); }
            'C' => { let n=p0.max(1); self.term.cursor_x=(self.term.cursor_x+n).min(self.term.cols-1); }
            'D' => { let n=p0.max(1); self.term.cursor_x=self.term.cursor_x.saturating_sub(n); }
            'H'|'f' => {
                self.term.cursor_y=(if p0==0{0}else{p0-1}).min(self.term.rows-1);
                self.term.cursor_x=(if p1==0{0}else{p1-1}).min(self.term.cols-1);
            }
            'J' => match p0 {
                0 => {
                    for x in self.term.cursor_x..self.term.cols { self.term.grid[self.term.cursor_y][x]=Cell::default(); }
                    for y in (self.term.cursor_y+1)..self.term.rows { self.term.grid[y]=vec![Cell::default();self.term.cols]; }
                }
                1 => {
                    for y in 0..self.term.cursor_y { self.term.grid[y]=vec![Cell::default();self.term.cols]; }
                    for x in 0..=self.term.cursor_x { self.term.grid[self.term.cursor_y][x]=Cell::default(); }
                }
                2|3 => { self.term.grid=vec![vec![Cell::default();self.term.cols];self.term.rows]; self.term.cursor_x=0; self.term.cursor_y=0; }
                _ => {}
            }
            'K' => match p0 {
                0 => { for x in self.term.cursor_x..self.term.cols { self.term.grid[self.term.cursor_y][x]=Cell::default(); } }
                1 => { for x in 0..=self.term.cursor_x { self.term.grid[self.term.cursor_y][x]=Cell::default(); } }
                2 => { self.term.grid[self.term.cursor_y]=vec![Cell::default();self.term.cols]; }
                _ => {}
            }
            'P' => {
                let n=p0.max(1); let row=self.term.cursor_y; let cx=self.term.cursor_x; let cols=self.term.cols;
                for i in cx..cols { self.term.grid[row][i]=if i+n<cols{self.term.grid[row][i+n]}else{Cell::default()}; }
            }
            'S' => { let n=p0.max(1); for _ in 0..n { self.term.scroll_up(); } }
            'T' => { let n=p0.max(1); for _ in 0..n { self.term.grid.pop(); self.term.grid.insert(0,vec![Cell::default();self.term.cols]); } }
            'm' => {
                if ps.is_empty()||(ps.len()==1&&ps[0]==0) {
                    self.term.cur_fg=Color::DEFAULT_FG; self.term.cur_bg=Color::DEFAULT_BG; self.term.cur_attrs=CellAttrs::default(); return;
                }
                let mut i=0;
                while i<ps.len() {
                    match ps[i] {
                        0  => { self.term.cur_fg=Color::DEFAULT_FG; self.term.cur_bg=Color::DEFAULT_BG; self.term.cur_attrs=CellAttrs::default(); }
                        1  => { self.term.cur_attrs.bold=true; }
                        2  => { self.term.cur_attrs.dim=true; }
                        3  => { self.term.cur_attrs.italic=true; }
                        4  => { self.term.cur_attrs.underline=true; }
                        5  => { self.term.cur_attrs.blink=true; }
                        7  => { self.term.cur_attrs.reverse=true; }
                        9  => { self.term.cur_attrs.strikethrough=true; }
                        22 => { self.term.cur_attrs.bold=false; self.term.cur_attrs.dim=false; }
                        23 => { self.term.cur_attrs.italic=false; }
                        24 => { self.term.cur_attrs.underline=false; }
                        27 => { self.term.cur_attrs.reverse=false; }
                        29 => { self.term.cur_attrs.strikethrough=false; }
                        30..=37 => { self.term.cur_fg=color256((ps[i]-30) as u8); }
                        38 => {
                            if ps.get(i+1).copied()==Some(5) {
                                if let Some(&idx)=ps.get(i+2) { self.term.cur_fg=color256(idx as u8); } i+=2;
                            } else if ps.get(i+1).copied()==Some(2) {
                                let r=ps.get(i+2).copied().unwrap_or(0) as u8;
                                let g=ps.get(i+3).copied().unwrap_or(0) as u8;
                                let b=ps.get(i+4).copied().unwrap_or(0) as u8;
                                self.term.cur_fg=Color::rgb(r,g,b); i+=4;
                            }
                        }
                        39 => { self.term.cur_fg=Color::DEFAULT_FG; }
                        40..=47 => { self.term.cur_bg=color256((ps[i]-40) as u8); }
                        48 => {
                            if ps.get(i+1).copied()==Some(5) {
                                if let Some(&idx)=ps.get(i+2) { self.term.cur_bg=color256(idx as u8); } i+=2;
                            } else if ps.get(i+1).copied()==Some(2) {
                                let r=ps.get(i+2).copied().unwrap_or(0) as u8;
                                let g=ps.get(i+3).copied().unwrap_or(0) as u8;
                                let b=ps.get(i+4).copied().unwrap_or(0) as u8;
                                self.term.cur_bg=Color::rgb(r,g,b); i+=4;
                            }
                        }
                        49 => { self.term.cur_bg=Color::DEFAULT_BG; }
                        90..=97  => { self.term.cur_fg=color256((ps[i]-90+8) as u8); }
                        100..=107 => { self.term.cur_bg=color256((ps[i]-100+8) as u8); }
                        _ => {}
                    }
                    i+=1;
                }
            }
            'l'|'h'|'r' => {}
            _ => {}
        }
    }
    fn hook(&mut self,_:&Params,_:&[u8],_:bool,_:char){}
    fn put(&mut self,_:u8){}
    fn unhook(&mut self){}
    fn osc_dispatch(&mut self,_:&[&[u8]],_:bool){}
    fn esc_dispatch(&mut self,_:&[u8],_:bool,byte:u8){
        if byte==b'M' && self.term.cursor_y>0 { self.term.cursor_y-=1; }
    }
}
