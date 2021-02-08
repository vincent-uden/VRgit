mod win;
mod git;
mod controller;

use ncurses::*;

use win::*;
use controller::Controller;

use std::env;
use std::path::PathBuf;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut controller = Controller::new(env::current_dir().unwrap());

    controller.init();

    if args.contains(&String::from("--log")) {
        controller.enable_logging();
    }

    controller.render();
    while controller.running() {

        controller.handle_key(getch());
        controller.render();

    }

    controller.close();
}
