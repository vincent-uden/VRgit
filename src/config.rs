use crate::mode::Action;

pub struct Config {
    pub stage_mode_key_map: Vec<(String, Action)>,
    pub commit_mode_key_map: Vec<(String, Action)>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            stage_mode_key_map: vec![
                ("j", Action::CursorDown),
                ("k", Action::CursorUp),
                ("q", Action::Exit),
                ("s", Action::StageFile),
                ("S", Action::StageAllFiles),
                ("u", Action::UnstageFile),
                ("c", Action::OpenCommitMode),
                ("?", Action::OpenHelpMode),
                ("p", Action::Push),
                ("<Esc>", Action::Exit),
            ]
            .iter()
            .map(|(s, a)| (String::from(*s), *a))
            .collect(),
            commit_mode_key_map: vec![
                ("-a", Action::ToggleCommitStageAll),
                ("-e", Action::ToggleCommitAllowEmpty),
                ("-v", Action::ToggleCommitVerbose),
                ("-n", Action::ToggleCommitDisableHooks),
                ("-R", Action::ToggleCommitResetAuthor),
                ("c", Action::OpenCommitMsgMode),
                ("<Esc>", Action::Exit),
            ]
            .iter()
            .map(|(s, a)| (String::from(*s), *a))
            .collect(),
        }
    }
}
