use bitflags::bitflags;
use itertools::izip;
use pancurses::{self, noecho, curs_set, cbreak, raw, use_default_colors, start_color, init_pair, endwin, COLOR_GREEN, COLOR_RED, COLOR_BLUE, COLOR_MAGENTA, COLOR_BLACK, COLOR_WHITE, COLOR_YELLOW, Attribute, COLOR_PAIR};

use std::ops;
use std::path::PathBuf;
use std::ptr;

bitflags! {
    pub struct TextStyle: u8 {
        const NORMAL      = 0b000000;
        const BOLD        = 0b000001;
        const ITALIC      = 0b000010;
        const UNDERLINE   = 0b000100;
    }
}

// static COLOR_BG: i16 = 256;
// static COLOR_FG: i16 = 257;

pub static COLOR_PAIR_DEFAULT: u32 = 1;
pub static COLOR_PAIR_H1: u32 = 2;
pub static COLOR_PAIR_H2: u32 = 3;
pub static COLOR_PAIR_H3: u32 = 4;
pub static COLOR_PAIR_SELECTED: u32 = 5;
pub static COLOR_PAIR_UNTRACKED: u32 = 6;
pub static COLOR_PAIR_SEP: u32 = 7;
pub static COLOR_PAIR_ENABLED: u32 = 8;

#[derive(Copy, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

pub struct Window {
    pub win: pancurses::Window,
}

pub struct ArgList {
    args: Vec<String>,
    arg_descs: Vec<String>,
    arg_long: Vec<String>,

    enabled: Vec<bool>,
}

pub struct FileList {
    pub files: Vec<PathBuf>,
    pub style: TextStyle,
    pub c_pair: u32,
}

pub struct KeyList {
    keys: Vec<String>,
    descs: Vec<String>,
}

pub struct Text {
    pub content: String,
    pub style: TextStyle,
    pub c_pair: u32,
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
    fn new() -> Self
    where
        Self: Sized;
    fn render(&self, win: &pancurses::Window,c: Coord);
    fn size(&self) -> Coord; // Assumes positive size, i32 is used for convenience
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x: x, y: y }
    }
}

impl ops::Add for Coord {
    type Output = Coord;
    fn add(self, other: Coord) -> Coord {
        Coord {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl ops::Sub for Coord {
    type Output = Coord;
    fn sub(self, other: Coord) -> Coord {
        Coord {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Window {
    pub fn new() -> Window {
        Window {
            win: pancurses::initscr(),
        }
    }

    pub fn init(&mut self) {
        self.win.keypad(true);
        noecho();
        curs_set(0);
        cbreak();
        raw();

        use_default_colors();
        start_color();
        init_pair(COLOR_PAIR_DEFAULT as i16, -1, -1);
        init_pair(COLOR_PAIR_H1 as i16, COLOR_GREEN, -1);
        init_pair(COLOR_PAIR_H2 as i16, COLOR_RED, -1);
        init_pair(COLOR_PAIR_H3 as i16, COLOR_BLUE, -1);
        init_pair(COLOR_PAIR_SELECTED as i16, COLOR_BLACK, COLOR_WHITE);
        init_pair(COLOR_PAIR_UNTRACKED as i16, COLOR_MAGENTA, -1);
        init_pair(COLOR_PAIR_SEP as i16, COLOR_BLACK, COLOR_BLUE);
        init_pair(COLOR_PAIR_ENABLED as i16, COLOR_YELLOW, -1);
    }

    pub fn render(&self) {
        self.win.refresh();
    }

    pub fn close(&self) {
        endwin();
    }

    pub fn get_size(&self) -> Coord {
        let (max_y, max_x) = self.win.get_max_yx();
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
        FileList {
            files: vec![],
            style: TextStyle::NORMAL,
            c_pair: COLOR_PAIR_DEFAULT,
        }
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        if self.bold() {
            win.attron(Attribute::Bold);
        }
        if self.italic() {
            win.attron(Attribute::Italic);
        }
        if self.underlined() {
            win.attron(Attribute::Underline);
        }
        win.attron(COLOR_PAIR(self.c_pair as u32));
        let mut i = 0;
        for path in &self.files {
            win.mvaddstr(c.y + i, c.x, &format!("{}\n", path.to_str().unwrap()));
            i += 1;
        }
        if self.bold() {
            win.attroff(Attribute::Bold);
        }
        if self.italic() {
            win.attroff(Attribute::Italic);
        }
        if self.underlined() {
            win.attroff(Attribute::Underline);
        }
        win.attroff(COLOR_PAIR(self.c_pair as u32));
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

    #[allow(dead_code)]
    pub fn get_enabled(&self) -> Vec<String> {
        self.arg_long
            .iter()
            .enumerate()
            .filter(|(i, _)| self.enabled[*i])
            .map(|(_, a)| String::from(a))
            .collect()
    }

    pub fn toggle(&mut self, arg: &str) {
        let index = self.args.iter().position(|x| arg == *x).unwrap();
        self.enabled[index] = !self.enabled[index];
    }
}

impl UiElement for ArgList {
    fn new() -> ArgList {
        ArgList {
            args: vec![],
            arg_descs: vec![],
            arg_long: vec![],
            enabled: vec![],
        }
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        for (i, arg, arg_d, arg_l, e) in izip!(
            0..self.args.len(),
            &self.args,
            &self.arg_descs,
            &self.arg_long,
            &self.enabled
        ) {
            win.attron(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            if *e {
                win.attron(Attribute::Bold);
            }
            win.mvaddstr(c.y + i as i32, c.x, arg);
            win.attroff(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            if *e {
                win.attroff(Attribute::Bold);
            }
            win.mvaddstr(c.y + i as i32, c.x + arg.len() as i32 + 1, arg_d);
            win.attron(COLOR_PAIR(if *e {
                COLOR_PAIR_ENABLED
            } else {
                COLOR_PAIR_H3
            }));
            win.mvaddstr(
                c.y + i as i32,
                c.x + arg.len() as i32 + arg_d.len() as i32 + 3,
                arg_l,
            );
            win.attroff(COLOR_PAIR(if *e {
                COLOR_PAIR_ENABLED
            } else {
                COLOR_PAIR_H3
            }));
            win.mvaddstr(
                c.y + i as i32,
                c.x + arg.len() as i32 + arg_d.len() as i32 + 2,
                "(",
            );
            win.mvaddstr(
                c.y + i as i32,
                c.x + arg.len() as i32 + arg_d.len() as i32 + arg_l.len() as i32 + 3,
                ")",
            );
        }
    }

    fn size(&self) -> Coord {
        let mut max_width = 0;
        for i in 0..self.args.len() {
            let width = format!(
                "{} {} ({})",
                self.args[i], self.arg_descs[i], self.arg_long[i]
            )
            .len();
            if width > max_width {
                max_width = width;
            }
        }

        Coord::new(max_width as i32, self.args.len() as i32)
    }
}

impl KeyList {
    pub fn push_key(&mut self, key: &str, desc: &str) {
        self.keys.push(String::from(key));
        self.descs.push(String::from(desc));
    }
}

impl UiElement for KeyList {
    fn new() -> KeyList {
        KeyList {
            keys: vec![],
            descs: vec![],
        }
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        for (i, key, desc) in izip!(0..self.keys.len(), &self.keys, &self.descs) {
            win.attron(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            win.mvaddstr(c.y + i as i32, c.x, &key);
            win.attroff(COLOR_PAIR(COLOR_PAIR_UNTRACKED));
            win.mvaddstr(c.y + i as i32, c.x + key.len() as i32 + 1, &desc);
        }
    }

    fn size(&self) -> Coord {
        let mut max_width = 0;
        for i in 0..self.keys.len() {
            let width = format!("{} {}", self.keys[i], self.descs[i]).len();
            if width > max_width {
                max_width = width;
            }
        }

        Coord::new(max_width as i32, self.keys.len() as i32)
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

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.content.len()
    }
}

impl UiElement for Text {
    fn new() -> Self {
        Text {
            content: String::from(""),
            style: TextStyle::NORMAL,
            c_pair: COLOR_PAIR_DEFAULT,
        }
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        win.attron(COLOR_PAIR(self.c_pair));
        if self.bold() {
            win.attron(Attribute::Bold);
        }
        if self.italic() {
            win.attron(Attribute::Italic);
        }
        if self.underlined() {
            win.attron(Attribute::Underline);
        }

        win.mvaddstr(c.y, c.x, &self.content);

        win.attroff(COLOR_PAIR(self.c_pair));
        if self.bold() {
            win.attroff(Attribute::Bold);
        }
        if self.italic() {
            win.attroff(Attribute::Italic);
        }
        if self.underlined() {
            win.attroff(Attribute::Underline);
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
        let mut lh = ListHeader {
            title: Text::new(),
            amount: Text::new(),
        };
        lh.title.c_pair = COLOR_PAIR_H3;
        lh.title.style = TextStyle::BOLD;
        lh
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        self.title.render(win, c);
        self.amount
            .render(win, Coord::new(c.x + self.title.size().x + 1, c.y));
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
}

impl UiElement for Layer {
    fn new() -> Self
    where
        Self: Sized,
    {
        Layer {
            elements: Vec::new(),
            positions: Vec::new(),
            visible: true,
        }
    }

    fn render(&self, win: &pancurses::Window, c: Coord) {
        if self.visible {
            for (i, e) in self.elements.iter().enumerate() {
                (*e).render(win, self.positions[i] + c);
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
