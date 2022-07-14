#[cfg(test)]
mod tests {
    use crate::mode::*;

    #[test]
    fn config_str_to_term_str_converts_space_esc() {
        let cfg1 = "<Esc>cc";
        let cfg2 = "<Space>Eg";

        assert_eq!(config_str_to_term_str(cfg1), format!("{}cc", 27 as char));
        assert_eq!(config_str_to_term_str(cfg2), String::from(" Eg"));
    }

    #[test]
    fn commit_mode_set_key_map() {
        let bindings = vec![
            ("k", Action::CursorUp),
            ("j", Action::CursorDown),
            ("gg", Action::CursorBufferStart),
            ("G", Action::CursorBufferEnd),
            ("<Esc>", Action::Exit),
        ].iter().map(|(c, a)| (String::from(*c), *a)).collect();

        let mut mode: StageMode = Mode::new();

        mode.set_key_map(bindings);

        assert_eq!(mode.handle_key('k' as i32), Action::CursorUp);
        assert_eq!(mode.handle_key('j' as i32), Action::CursorDown);
        assert_eq!(mode.handle_key('G' as i32), Action::CursorBufferEnd);

        assert_eq!(mode.handle_key('g' as i32), Action::Matching);
        assert_eq!(mode.handle_key('g' as i32), Action::CursorBufferStart);

        assert_eq!(mode.handle_key('g' as i32), Action::Matching);
        assert_eq!(mode.handle_key('h' as i32), Action::NoMatch);

        assert_eq!(mode.handle_key('h' as i32), Action::NoMatch);

        assert_eq!(mode.handle_key(27), Action::Exit);
    }
}
