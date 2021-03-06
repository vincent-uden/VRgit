
use ncurses::*;

use crate::win::*;
use crate::git::Git;

use std::path::{PathBuf};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[derive(PartialEq,Debug)]
enum OpenPanel {
    STAGING,
    COMMITING,
    COMMITMSG,
}

#[allow(dead_code)]
fn ctrl(key_code: i32) -> i32 {
    key_code & 0x1f
}

#[allow(dead_code)]
fn ctrl_c(key: char) -> i32 {
    ctrl(key as u8 as i32)
}

fn str_to_char_arr(s: &str) -> Vec<i32> {
    let mut output = Vec::with_capacity(s.len());
    for c in s.chars() {
        output.push(c as u8 as i32);
    }
    output
}

fn char_arr_to_str(arr: &Vec<i32>) -> String {
    let mut output = String::new();
    for c in arr {
        output.push((*c) as u8 as char);
    }
    output
}

pub struct Controller {
    running: bool,
    last_char: char,

    key_chord: Vec<i32>,
    bound_chords: Vec<Vec<i32>>,
    bound_args: Vec<Vec<i32>>,

    git: Git,
    win: Window,

    status_layer: Layer,
    pre_commit_layer: Layer,
    commit_msg_layer: Layer,

    cursor: Coord,

    fl1_pos: Coord,
    fl2_pos: Coord,
    fl3_pos: Coord,

    fl1_vec: Vec<PathBuf>,
    fl2_vec: Vec<PathBuf>,
    fl3_vec: Vec<PathBuf>,

    open_panel: OpenPanel,
    enabled_commit_args: HashSet<String>,

    debug_string: String,
    commit_msg: String,

    log_file: Option<File>,

    push_status: String,

}

impl Controller {
    pub fn new(path: PathBuf) -> Controller {
        Controller { 
            running: true, 
            last_char: ' ',
            key_chord: Vec::new(),
            bound_chords: Vec::new(),
            bound_args: Vec::new(),
            git: Git::new(path), 
            win: Window::new(), 
            status_layer: Layer::new(), 
            pre_commit_layer: Layer::new(),
            commit_msg_layer: Layer::new(),
            cursor: Coord::new(0, 0),
            fl1_pos: Coord::new(0, 0),
            fl2_pos: Coord::new(0, 0),
            fl3_pos: Coord::new(0, 0),
            fl1_vec: Vec::new(),
            fl2_vec: Vec::new(),
            fl3_vec: Vec::new(),
            open_panel: OpenPanel::STAGING,
            enabled_commit_args: HashSet::new(),
            debug_string: String::new(),
            commit_msg: String::new(),
            log_file: None,
            push_status: String::from(""),
        }
    }

    pub fn init(&mut self) {
        self.win.init();

        self.update_status_layer();
        self.update_pre_commit_layer();
        self.update_commit_msg_layer();

        self.cursor.x = 2;
        self.cursor.y = 2;

        self.bound_chords = vec![
            str_to_char_arr("j"),
            str_to_char_arr("k"),
            str_to_char_arr("q"),
            str_to_char_arr("s"),
            str_to_char_arr("u"),
            str_to_char_arr("c"),
            str_to_char_arr("p"),
        ];

        self.bound_args = vec![
            str_to_char_arr("-a"),
            str_to_char_arr("-e"),
            str_to_char_arr("-v"),
            str_to_char_arr("-n"),
            str_to_char_arr("-R"),
            str_to_char_arr("c"),
            vec![ 27 ],
        ];
    }

    pub fn enable_logging(&mut self) {
        self.log_file = Some(File::create("debug.log").unwrap());
    }

    pub fn render(&self) {
        clear();
        if self.open_panel == OpenPanel::STAGING {
            self.status_layer.render(Coord::new(0, 0));
        }
        if self.open_panel == OpenPanel::COMMITING {
            self.status_layer.render(Coord::new(0, 0));
            self.pre_commit_layer.render(Coord::new(0, self.win.get_size().y - self.pre_commit_layer.size().y - 1));
        }
        if self.open_panel == OpenPanel::COMMITMSG {
            self.commit_msg_layer.render(Coord::new(0, 0));
        }

        let mut i = 0;
        for thing in &self.enabled_commit_args {
            mvaddstr(20 + i, 20, thing);
            i += 1;
        }

        if self.open_panel != OpenPanel::COMMITMSG {
            let on_cursor = mvinch(self.cursor.y, self.cursor.x);
            // Mask out all color bits and apply the "selected" colors
            mvaddch(self.cursor.y, self.cursor.x, on_cursor & (!COLOR_PAIR(0xFF)) | COLOR_PAIR(COLOR_PAIR_SELECTED));
        }
        mvaddch(15, 0, self.last_char as u32);
        mvaddstr(20, 0, &format!("{:?}", char_arr_to_str(&self.key_chord)));
        mvaddstr(16, 0, &format!("{:?}", self.open_panel));
        mvaddstr(18, 0, &format!("Key chord: {}", char_arr_to_str(&self.key_chord)));
        mvaddstr(19, 0, &format!("Debug msg: {:?}", self.debug_string));
        
        if self.push_status != String::from("") {
            let pos = Coord::new(0, self.fl3_pos.y + self.fl3_vec.len() as i32 + 1);
            let mut push_msg: Text = UiElement::new();
            push_msg.content = self.push_status.clone();
            push_msg.style = TextStyle::BOLD;
            push_msg.c_pair = COLOR_PAIR_H1;
            push_msg.render(pos);
        }

        self.win.render();
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn close(&mut self) {
        self.running = false;
        self.win.close();
    }

    pub fn handle_key(&mut self, key: i32) {
        self.last_char = key as u8 as char;
        self.push_status = String::from("");

        self.key_chord.push(key);

        let mut matched = true;

        self.debug_string.clear();

        match self.open_panel {
        OpenPanel::STAGING => 
            if self.key_chord == vec![ 'j' as i32 ] {
                self.cursor_move(1);
            } else if self.key_chord == vec![ 'k' as i32 ] {
                self.cursor_move(-1);
            } else if self.key_chord == vec![ 'q' as i32 ] {
                self.close();
            } else if self.key_chord == vec![ 's' as i32 ] {
                let file = self.get_file();
                match file {
                    Some(p) => self.git.stage_file(p),
                    None    => (),
                }
            } else if self.key_chord == vec![ 'u' as i32 ] {
                let file = self.get_file();
                match file {
                    Some(p) => self.git.unstage_file(p),
                    None    => (),
                }
            } else if self.key_chord == vec![ 'c' as i32 ] {
                self.open_panel = OpenPanel::COMMITING;
            } else if self.key_chord == vec![ 'p' as i32 ] {
                self.debug_string = String::from("Push complete");
                self.render_push_start();
                let result = self.git.push();
                match self.log_file {
                    Some(ref mut file) => {
                        match file.write(result.as_bytes()) {
                            Err(_) => println!("Error while writing to debug file"),
                            _ => {},
                        }
                    },
                    None => {
                    },
                }
                self.push_status = result;
            } else {
                matched = false;
            },
        OpenPanel::COMMITING => 
            if self.key_chord == str_to_char_arr("-a") {
                if !self.enabled_commit_args.insert("-a".to_string()) {
                    self.enabled_commit_args.remove("-a");
                }
            } else if self.key_chord == str_to_char_arr("-e") {
                if !self.enabled_commit_args.insert("-e".to_string()) {
                    self.enabled_commit_args.remove("-e");
                }
            } else if self.key_chord == str_to_char_arr("-v") {
                if !self.enabled_commit_args.insert("-v".to_string()) {
                    self.enabled_commit_args.remove("-v");
                }
            } else if self.key_chord == str_to_char_arr("-n") {
                if !self.enabled_commit_args.insert("-n".to_string()) {
                    self.enabled_commit_args.remove("-n");
                }
            } else if self.key_chord == str_to_char_arr("-R") {
                if !self.enabled_commit_args.insert("-R".to_string()) {
                    self.enabled_commit_args.remove("-R");
                }
            } else if self.key_chord == str_to_char_arr("c")  {
                self.open_panel = OpenPanel::COMMITMSG;
                self.update_commit_msg_layer();
            } else if self.key_chord == vec![ 27 ] {
                self.open_panel = OpenPanel::STAGING;
            } else {
                matched = false;
            },
        OpenPanel::COMMITMSG =>
            if self.key_chord == vec![ 27 ] {
                self.open_panel = OpenPanel::COMMITING;
            } else if self.key_chord == vec![ 263 ] {
                self.commit_msg.pop();
                self.update_commit_msg_layer();
                matched = false;
            } else if self.last_char == '\n' {
                self.git.commit(self.enabled_commit_args.clone().into_iter().collect(), self.commit_msg.clone());
                self.open_panel = OpenPanel::STAGING;
            } else {
                // TODO: Handle åäö, they fuck everything up
                self.commit_msg.push(self.last_char);
                self.update_commit_msg_layer();
                matched = false;
            },
        }

            //self.debug_string = self.commit_msg.clone();
        if matched {
            self.key_chord.clear();
            self.update_status_layer();
            self.update_pre_commit_layer();

            // self.debug_string = String::from("MATCH!");
        } else {
            // self.debug_string = String::from("NO MATCH!");
            let mut starting = false;
            let chord_len = self.key_chord.len();
            match self.open_panel {
                OpenPanel::STAGING => {
                    for chord in &self.bound_chords {
                        if chord.len() >= chord_len {
                            if chord[0..chord_len] == self.key_chord[..] {
                                starting = true;
                            }
                        }
                    }
                },
                OpenPanel::COMMITING => {
                    for chord in &self.bound_args {
                        if chord.len() >= chord_len {
                            if chord[0..chord_len] == self.key_chord[..] {
                                starting = true;
                            }
                        }
                    }
                },
                OpenPanel::COMMITMSG => {
                }
            }

            if !starting {
                self.key_chord.clear();
            }
        }
    }

    fn cursor_move(&mut self, amount: i32) {
        self.cursor.y += amount;
    }

    fn get_file(&self) -> Option<&PathBuf> {
        // Untracked file
        if self.cursor.y >= self.fl1_pos.y && 
           self.cursor.y < self.fl1_pos.y + self.fl1_vec.len() as i32 && 
           self.fl1_vec.len() > 0 {
               return Some(&self.fl1_vec[(self.cursor.y - self.fl1_pos.y) as usize]);
        }
        // Staged file
        if self.cursor.y >= self.fl2_pos.y && 
           self.cursor.y < self.fl2_pos.y + self.fl2_vec.len() as i32 && 
           self.fl2_vec.len() > 0 {
               return Some(&self.fl2_vec[(self.cursor.y - self.fl2_pos.y) as usize]);
        }
        // Unstaged file
        if self.cursor.y >= self.fl3_pos.y && 
           self.cursor.y < self.fl3_pos.y + self.fl3_vec.len() as i32 && 
           self.fl3_vec.len() > 0 {
               return Some(&self.fl3_vec[(self.cursor.y - self.fl3_pos.y) as usize]);
        }

        None
    }

    fn update_status_layer(&mut self) {
        self.status_layer = Layer::new();

        let mut branch_title: Text = UiElement::new();
        let mut branch_name: Text = UiElement::new();
        let mut last_commit_msg: Text = UiElement::new();
        let mut fl1: FileList = UiElement::new();
        let mut fl2: FileList = UiElement::new();
        let mut fl3: FileList = UiElement::new();
        let mut untracked_header: ListHeader = UiElement::new();
        let mut staged_header: ListHeader = UiElement::new();
        let mut unstaged_header : ListHeader = UiElement::new();

        branch_title.content = String::from("Head:    ");
        branch_name.content = self.git.branch_name();
        branch_name.c_pair = COLOR_PAIR_H3;
        last_commit_msg.content = self.git.last_commit_msg();

        fl1.files = self.git.untracked();
        fl1.c_pair = COLOR_PAIR_UNTRACKED;
        fl2.files = self.git.staged();
        fl2.style = TextStyle::BOLD;
        fl3.files = self.git.unstaged();
        fl3.style = TextStyle::BOLD;

        untracked_header.set_title(String::from("Untracked Files"));
        untracked_header.set_amount(fl1.size().y);
        staged_header.set_title(String::from("Staged changes"));
        staged_header.set_amount(fl2.size().y);
        unstaged_header.set_title(String::from("Unstaged changes"));
        unstaged_header.set_amount(fl3.size().y);

        self.fl1_vec = self.git.untracked();
        self.fl2_vec = self.git.staged();
        self.fl3_vec = self.git.unstaged();

        let s1 = branch_title.size();
        let s2 = branch_name.size();
        let s3 = Coord::new(0,4 + fl1.size().y);
        let s4 = Coord::new(0, s3.y + 2 + fl2.size().y);

        println!("{}", fl1.size().y);

        self.fl1_pos = Coord::new(2, 3);
        self.fl2_pos = Coord::new(2, 1 + s3.y);
        self.fl3_pos = s4 + Coord::new(2, 1);

        self.status_layer.push(Box::new(branch_title), Coord::new(0, 0));
        self.status_layer.push(Box::new(branch_name), Coord::new(s1.x, 0));
        self.status_layer.push(Box::new(last_commit_msg), Coord::new(s1.x + s2.x, 0));
        self.status_layer.push(Box::new(untracked_header), Coord::new(0, 2));
        self.status_layer.push(Box::new(fl1), self.fl1_pos);
        self.status_layer.push(Box::new(staged_header), s3);
        self.status_layer.push(Box::new(fl2), self.fl2_pos);
        self.status_layer.push(Box::new(unstaged_header), s4);
        self.status_layer.push(Box::new(fl3), self.fl3_pos);
    }

    fn update_pre_commit_layer(&mut self) {
        self.pre_commit_layer = Layer::new();

        let mut separator: Text = UiElement::new();
        let mut arg_header: Text = UiElement::new();
        let mut arg_list: ArgList = UiElement::new();
        let mut commit_header: Text = UiElement::new();
        let mut commit_c: Text = UiElement::new();
        let mut commit_text: Text = UiElement::new();

        separator.content = String::from("=".repeat(self.win.get_size().x as usize));
        separator.c_pair = COLOR_PAIR_SEP;
        arg_header.content = String::from("Arguments");
        arg_header.c_pair = COLOR_PAIR_H3;
        commit_header.content = String::from("Create");
        commit_header.c_pair = COLOR_PAIR_H3;
        commit_c.content = String::from("c");
        commit_c.c_pair = COLOR_PAIR_UNTRACKED;
        commit_text.content = String::from("Commit");

        arg_list.push_arg("-a", "Stage all modified and deleted files", "--all");
        arg_list.push_arg("-e", "Allow empty commit", "--allow-empty");
        arg_list.push_arg("-v", "Show diff of changes to be commited", "--verbose");
        arg_list.push_arg("-n", "Disable hooks", "--no-verify");
        arg_list.push_arg("-R", "Claim authorship and reset author date", "--reset-author");

        for arg in self.enabled_commit_args.iter() {
            arg_list.toggle(&(arg.clone()));
        }

        self.pre_commit_layer.push(Box::new(separator), Coord::new(0, 0));
        self.pre_commit_layer.push(Box::new(arg_header), Coord::new(0, 2));
        self.pre_commit_layer.push(Box::new(commit_header), Coord::new(0, 4 + arg_list.size().y));
        self.pre_commit_layer.push(Box::new(commit_c), Coord::new(1, 5 + arg_list.size().y));
        self.pre_commit_layer.push(Box::new(commit_text), Coord::new(3, 5 + arg_list.size().y));
        self.pre_commit_layer.push(Box::new(arg_list), Coord::new(1, 3));

        // TODO: Add popup for entering commit message
    }

    fn update_commit_msg_layer(&mut self) {
        self.commit_msg_layer = Layer::new();

        let mut header: Text = UiElement::new();
        let mut message: Text = UiElement::new();
        let mut fl1: FileList = UiElement::new();
        let mut changes_header: Text = UiElement::new();

        header.content = String::from("Please enter the commit message for your changes.\n >  ");
        header.c_pair = COLOR_PAIR_H3;
        message.content = self.commit_msg.clone();
        message.c_pair = COLOR_PAIR_H1;
        if self.enabled_commit_args.contains("-a") {
            fl1.files = [self.git.staged(), self.git.unstaged()].concat();
        } else {
            fl1.files = self.git.staged();
        }
        fl1.c_pair = COLOR_PAIR_UNTRACKED;
        changes_header.content = String::from("Changed to be committed:");
        changes_header.c_pair = COLOR_PAIR_H3;
        changes_header.style = TextStyle::BOLD;

        self.commit_msg_layer.push(Box::new(header), Coord::new(0, 0));
        self.commit_msg_layer.push(Box::new(message), Coord::new(3, 1));
        self.commit_msg_layer.push(Box::new(fl1), Coord::new(1, 4));
        self.commit_msg_layer.push(Box::new(changes_header), Coord::new(0, 3));
    }

    fn render_push_start(&self) {
        let pos = Coord::new(0, self.fl3_pos.y + self.fl3_vec.len() as i32 + 1);
        let mut push_msg: Text = UiElement::new();
        push_msg.content = String::from("Pushing...");
        push_msg.style = TextStyle::BOLD;
        push_msg.c_pair = COLOR_PAIR_SELECTED;

        push_msg.render(pos);
        refresh();
    }

}
