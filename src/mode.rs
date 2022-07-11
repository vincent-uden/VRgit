use crate::util::*;

use std::char;
use std::iter::zip;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    CursorBufferEnd,
    CursorBufferStart,
    CursorDown,
    CursorUp,
    Error,
    Exit,
    Matching,
    NoMatch,
    OpenCommitMode,
    OpenCommitMsgMode,
    OpenHelpMode,
    Push,
    StageAllFiles,
    StageFile,
    ToggleCommitAllowEmpty,
    ToggleCommitDisableHooks,
    ToggleCommitResetAuthor,
    ToggleCommitStageAll,
    ToggleCommitVerbose,
    UnstageFile,
}

pub trait Mode {
    fn new() -> Self
    where
        Self: Sized;
    fn handle_key(&mut self, key: i32) -> Action;

    fn get_bound_chords(&self) -> Vec<String>;
    fn get_bound_actions(&self) -> Vec<Action>;
    // TODO: Think about inserting bindings during runtime one at a time. Is
    // that a reasonable thing to do?
    fn set_key_map(&mut self, bindings: Vec<(&str, Action)>);

}

pub fn config_str_to_term_str(ch: &str) -> String {
    // TODO: Complete with conversion for Ctrl
    let mut chord = String::from(ch);
    chord = chord.replace("<Esc>", &format!("{}", 27 as char));
    chord = chord.replace("<Space>", " ");

    return chord;
}


pub struct StageMode {
    keys: Vec<String>,
    bound_fns: Vec<Action>,
    chord: String,
    longest_chord: usize,
}

impl Mode for StageMode {
    fn new() -> Self
    where
        Self: Sized,
    {
        StageMode {
            keys: Vec::new(),
            bound_fns: Vec::new(),
            chord: String::new(),
            longest_chord: 0,
        }
    }

    fn handle_key(&mut self, key: i32) -> Action {
        // Not a key press
        if key < 0 {
            self.chord.clear();
            return Action::Error;
        }

        let pressed = char::from_u32(key as u32);
        // Not all u32s are valid keys
        if pressed.is_none() {
            self.chord.clear();
            return Action::Error;
        }

        self.chord.push(pressed.unwrap());
        // No matching key binding
        if self.chord.chars().count() > self.longest_chord {
            self.chord.clear();
            return Action::NoMatch;
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
            return Action::Matching;
        } else { 
            self.chord.clear();
            return Action::NoMatch;
        }
    }

    fn get_bound_chords(&self) -> Vec<String> {
        return self.keys.clone();
    }

    fn get_bound_actions(&self) -> Vec<Action> {
        return self.bound_fns.clone();
    }

    fn set_key_map(&mut self, bindings: Vec<(&str, Action)>) {
        self.longest_chord = 0;
        for (ch, fun) in bindings {
            if ch.chars().count() > self.longest_chord {
                self.longest_chord = ch.chars().count();
            }
            self.keys.push(config_str_to_term_str(ch));
            self.bound_fns.push(fun);
        }
    }
}
