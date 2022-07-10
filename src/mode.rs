use crate::util::*;

use std::char;
use std::iter::zip;

pub trait Mode {
    fn new() -> Self
    where
        Self: Sized;
    fn handle_key(&mut self, key: i32) -> fn();

    // TODO: Think about inserting bindings during runtime one at a time. Is
    // that a reasonable thing to do?
    fn set_key_map(&mut self, bindings: Vec<(&str, fn())>);
}

fn blank() {}

fn config_str_to_term_str(ch: &str) -> String {
    // TODO: Complete with conversion for Ctrl
    let mut chord = String::from(ch);
    chord = chord.replace("<Esc>", &format!("{}", 27 as char));
    chord = chord.replace("<Space>", " ");

    return chord;
}

pub struct CommitMode {
    keys: Vec<String>,
    bound_fns: Vec<fn()>,
    chord: String,
    longest_chord: usize,
    error_func: fn(),
}

impl Mode for CommitMode {
    fn new() -> Self
    where
        Self: Sized,
    {
        CommitMode {
            keys: Vec::new(),
            bound_fns: Vec::new(),
            chord: String::new(),
            longest_chord: 0,
            error_func: blank,
        }
    }

    fn handle_key(&mut self, key: i32) -> fn() {
        // Not a key press
        if key < 0 {
            return self.error_func;
        }

        let pressed = char::from_u32(key as u32);
        // Not all u32s are valid keys
        if pressed.is_none() {
            return self.error_func;
        }

        self.chord.push(pressed.unwrap());
        // No matching key binding
        if self.chord.chars().count() > self.longest_chord {
            return self.error_func;
        }

        let mut potential_match = false;
        // Attempt to find matching key binding
        for (ch, fun) in zip(&self.keys, &self.bound_fns) {
            if ch == &self.chord {
                self.chord.clear();
                return *fun;
            } else if ch.starts_with(&self.chord) {
                potential_match = true;
            }
        }

        // Return self.error_func means that there is no point in trying to
        // investigate the current chord any further
        if potential_match { 
            return blank;
        } else { 
            return self.error_func;
        }
    }

    fn set_key_map(&mut self, bindings: Vec<(&str, fn())>) {
        for (ch, fun) in &bindings {
            self.keys.push(config_str_to_term_str(ch));
            self.bound_fns.push(*fun);
        }
    }
}
