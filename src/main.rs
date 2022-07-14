mod config;
mod controller;
mod git;
mod mode;
mod tests;
mod util;
mod win;

use pancurses::*;

#[cfg(target_os = "linux")]
use ncurses::{set_escdelay};

use controller::Controller;

use std::env::{self, consts};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut controller = Controller::new(env::current_dir().unwrap());

    controller.init();

    if consts::OS != "windows" {
        set_escdelay(1);
    }

    if args.contains(&String::from("--log")) {
        controller.enable_logging();
    }

    controller.render();
    while controller.running() {
        controller.handle_key(match controller.win.win.getch() {
            Some(Input::Character(c)) => c as i32,
            _ => 0,
        });
        controller.render();
    }

    controller.close();
}
