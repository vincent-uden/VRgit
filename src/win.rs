use ncurses::*;
use bitflags::bitflags;
use itertools::izip;

use std::ptr;
use std::path::PathBuf;
use std::ops;

bitflags! {
    pub struct TextStyle: u8 {
        const NORMAL      = 0b000000;
        const BOLD        = 0b000001;
        const ITALIC      = 0b000010;
        const UNDERLINE   = 0b000100;
    }
}

static COLOR_BG: i16 = 256;
static COLOR_FG: i16 = 257;

pub static COLOR_PAIR_DEFAULT: i16 = 1;
pub static COLOR_PAIR_H1: i16 = 2;
pub static COLOR_PAIR_H2: i16 = 3;
pub static COLOR_PAIR_H3: i16 = 4;
pub static COLOR_PAIR_SELECTED: i16 = 5;
pub static COLOR_PAIR_UNTRACKED: i16 = 6;
pub static COLOR_PAIR_SEP: i16 = 7;
pub static COLOR_PAIR_ENABLED: i16 = 8;

#[derive(Copy, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

pub struct Window {
    title: String,
    win: WINDOW,
} 

pub struct ArgList {
    args:      Vec<String>,
    arg_descs: Vec<String>,
    arg_long:  Vec<String>,

    enabled:   Vec<bool>,
}

pub struct FileList {
    pub files: Vec<PathBuf>,
    pub style: TextStyle,
    pub c_pair: i16,
}

pub struct Text {
    pub content: String,
    pub style: TextStyle,
    pub c_pair: i16,
}

pub struct ListHeader {
    title: Text,
    amount: Text,
}

pub struct Layer {
    elements: Vec<Box<dyn UiElement>>,
    positions: Vec<Coord>,
    pub visible: bool,
}

pub trait UiElement {
    fn new() -> Self where Self: Sized;
    fn render(&self, c: Coord);
    fn size(&self) -> Coord; // Assumes positive size, i32 is used for convenience
}

impl Coord {
    pub fn new(x: i32, y:i32) -> Coord {
        Coord { x: x, y: y}
    }
}

impl ops::Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord { x: self.x + other.x, y: self.y + other.y }
    }
}

impl ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Window {
    pub fn new(title: &str) -> Window {
        Window {
            title: String::from(title),
            win: ptr::null_mut(),
        }
    }

    pub fn init(&mut self) {
        self.win = initscr();
        keypad(self.win, true);
        noecho();
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        cbreak();
        raw();
        set_escdelay(1);
        setlocale(LcCategory::all, "");

        use_default_colors();
        start_color();
        init_pair(COLOR_PAIR_DEFAULT, -1, -1);
        init_pair(COLOR_PAIR_H1, COLOR_GREEN, -1);
        init_pair(COLOR_PAIR_H2, COLOR_RED, -1);
        init_pair(COLOR_PAIR_H3, COLOR_BLUE, -1);
        init_pair(COLOR_PAIR_SELECTED, COLOR_BLACK, COLOR_WHITE);
        init_pair(COLOR_PAIR_UNTRACKED, COLOR_MAGENTA, -1);
        init_pair(COLOR_PAIR_SEP, COLOR_BLACK, COLOR_BLUE);
        init_pair(COLOR_PAIR_ENABLED, COLOR_YELLOW, -1);
    }

    pub fn render(&self) {
        refresh();
    }

    pub fn close(&self) {
        endwin();
    }

    pub fn get_size(&self) -> Coord {
        let mut max_x = 0;
        let mut max_y = 0;
        getmaxyx(self.win, &mut max_y, &mut max_x);
        Coord::new(max_x, max_y)
    }
}

impl FileList {
    pub fn len(&self) -> usize {
        self.files.len()
    }

    fn bold(&self) -> bool {
        self.style.intersects(TextStyle::BOLD)
    }

    fn italic(&self) -> bool {
        self.style.intersects(TextStyle::ITALIC)
    }

    fn underlined(&self) -> bool {
        self.style.intersects(TextStyle::UNDERLINE)
    }
}

impl UiElement for FileList {
    fn new() -> FileList {
        FileList { files: vec!() , style: TextStyle::NORMAL, c_pair: COLOR_PAIR_DEFAULT }
    }

    fn render(&self, c: Coord) {
        if self.bold() {
            attron(A_BOLD());
        }
        if self.italic() {
            attron(A_ITALIC());
        }
        if self.underlined() {
            attron(A_UNDERLINE());
        }
        attron(COLOR_PAIR(self.c_pair));
        let mut i = 0;
        for path in &self.files {
            mvaddstr(c.y + i, c.x, &format!("{}\n", path.to_str().unwrap()));
            i += 1;
        }
        if self.bold() {
            attroff(A_BOLD());
        }
        if self.italic() {
            attroff(A_ITALIC());
        }
        if self.underlined() {
            attroff(A_UNDERLINE());
        }
        attroff(COLOR_PAIR(self.c_pair));
    }

    fn size(&self) -> Coord {
        let mut biggest = 0;
        let mut l;
        for p in &self.files {
            l = p.to_str().unwrap().len();
            if l > biggest {
                biggest = l;
            }
        }
        Coord::new(biggest as i32, self.len() as i32)
    }
}

impl ArgList {
    pub fn push_arg(&mut self, arg: &str, arg_desc: &str, arg_long: &str) {
        self.args.push(String::from(arg));
        self.arg_descs.push(String::from(arg_desc));
        self.arg_long.push(String::from(arg_long));

        self.enabled.push(false);
    }

    pub fn get_enabled(&self) -> Vec<String> {
        self.arg_long.iter().enumerate().filter(|(i,a)| self.enabled[*i]).map(|(i,a)| String::from(a)).collect()
    }

    pub fn toggle(&mut self, arg: &str) {
        let index = self.args.iter().position(|x| arg == *x).unwrap();
        self.enabled[index] = !self.enabled[index];
    }
}

impl UiElement for ArgList {
    fn new() -> ArgList {
        ArgList { args: vec![], arg_descs: vec![], arg_long: vec![], enabled: vec![] }
    }

    fn render(&self, c: Coord) {
        for (i, arg, arg_d, arg_l, e) in izip!(0..self.args.len(), &self.args, &self.arg_descs, &self.arg_long, &self.enabled) {
            attron(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            if *e {
                attron(A_BOLD());
            }
            mvaddstr(c.y + i as i32, c.x, arg);
            attroff(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            if *e {
                attroff(A_BOLD());
            }
            mvaddstr(c.y + i as i32, c.x + arg.len() as i32 + 1, arg_d);
            attron(COLOR_PAIR(if *e { COLOR_PAIR_ENABLED } else { COLOR_PAIR_H3 }));
            mvaddstr(c.y + i as i32, c.x + arg.len() as i32+ arg_d.len() as i32 + 3, arg_l);
            attroff(COLOR_PAIR(if *e { COLOR_PAIR_ENABLED } else { COLOR_PAIR_H3 }));
            mvaddstr(c.y + i as i32, c.x + arg.len() as i32+ arg_d.len() as i32 + 2, "(");
            mvaddstr(c.y + i as i32, c.x + arg.len() as i32+ arg_d.len() as i32 + arg_l.len() as i32 + 3, ")");
        }
    }

    fn size(&self) -> Coord {
        let mut max_width = 0;
        for i in 0..self.args.len() {
            let width = format!("{} {} ({})", self.args[i], self.arg_descs[i], self.arg_long[i]).len();
            if width > max_width {
                max_width = width;
            }
        }

        Coord::new(max_width as i32, self.args.len() as i32)
    }
}

impl Text {
    fn bold(&self) -> bool {
        self.style.intersects(TextStyle::BOLD)
    }

    fn italic(&self) -> bool {
        self.style.intersects(TextStyle::ITALIC)
    }

    fn underlined(&self) -> bool {
        self.style.intersects(TextStyle::UNDERLINE)
    }

    fn len(&self) -> usize {
        self.content.len()
    }
}

impl UiElement for Text {
    fn new() -> Self {
        Text { content: String::from(""), style: TextStyle::NORMAL, c_pair: COLOR_PAIR_DEFAULT }
    }

    fn render(&self, c: Coord) {
        attron(COLOR_PAIR(self.c_pair));
        if self.bold() {
            attron(A_BOLD());
        }
        if self.italic() {
            attron(A_ITALIC());
        }
        if self.underlined() {
            attron(A_UNDERLINE());
        }

        mvaddstr(c.y, c.x, &self.content);

        attroff(COLOR_PAIR(self.c_pair));
        if self.bold() {
            attroff(A_BOLD());
        }
        if self.italic() {
            attroff(A_ITALIC());
        }
        if self.underlined() {
            attroff(A_UNDERLINE());
        }
    }

    fn size(&self) -> Coord {
        Coord::new(self.content.len() as i32, (self.content.len() > 0) as i32)
    }
}

impl ListHeader {
    pub fn set_title(&mut self, title: String) {
        self.title.content = title;
    }
    
    pub fn set_amount(&mut self, amount: i32) {
        self.amount.content = format!("({})", amount);
    }
}

impl UiElement for ListHeader {
    fn new() -> Self {
        let mut lh = ListHeader { title: Text::new(), amount: Text::new() };
        lh.title.c_pair = COLOR_PAIR_H3;
        lh.title.style = TextStyle::BOLD;
        lh
    }

    fn render(&self, c: Coord) {
        self.title.render(c);
        self.amount.render(Coord::new(c.x + self.title.size().x + 1, c.y));
    }

    fn size(&self) -> Coord {
        Coord::new((self.title.size().x + self.amount.size().x + 1) as i32, 1)
    }
}

impl Layer {
    pub fn push(&mut self, item: Box<dyn UiElement>, c: Coord) {
        self.elements.push(item);
        self.positions.push(c);
    }

    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

impl UiElement for Layer {
    fn new() -> Self where Self: Sized {
        Layer { elements: Vec::new(), positions: Vec::new(), visible: true }
    }

    fn render(&self, c: Coord) {
        if self.visible {
            for (i, e) in self.elements.iter().enumerate() {
                (*e).render(self.positions[i] + c);
            }
        }
    }

    fn size(&self) -> Coord {
        let mut min_x = 0;
        let mut min_y = 0;
        let mut max_x = 0;
        let mut max_y = 0;

        for (i, e) in self.elements.iter().enumerate() {
            if self.positions[i].x < min_x {
                min_x = self.positions[i].x;
            }
            if self.positions[i].y < min_y {
                min_y = self.positions[i].y;
            }
            if self.positions[i].x + (*e).size().x > max_x {
                max_x = self.positions[i].x + (*e).size().x;
            }
            if self.positions[i].y + (*e).size().y > max_y {
                max_y = self.positions[i].y + (*e).size().y;
            }
        }

        Coord::new(max_x - min_x, max_y - min_y)
    }
}
