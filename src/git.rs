use std::path::PathBuf;
use std::process::Command;
use std::collections::HashSet;

pub struct Git {
    work_dir: PathBuf,
}

impl Git {
    pub fn new(path: PathBuf) -> Git {
        Git { work_dir: path }
    }

    pub fn w_dir(&self) -> String {
        format!("-C {}", self.work_dir.to_str().unwrap())
    }

    pub fn untracked(&self) -> Vec<PathBuf> {
        let child = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .output()
            .expect("Git failed");
        
        (&String::from_utf8_lossy(&child.stdout)).lines().map(|l| PathBuf::from(l)).collect()
    }

    pub fn staged(&self) -> Vec<PathBuf> {
        let child = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("status")
            .arg("--porcelain")
            .output()
            .expect("Couldn't run git status");
        
        (String::from_utf8_lossy(&child.stdout)).lines()
            .filter(|l| l.chars().nth(1).unwrap() == ' ')
            .map(|p| PathBuf::from(&p[3..])).collect()
    }

    pub fn unstaged(&self) -> Vec<PathBuf> {
        let child = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("status")
            .arg("--porcelain")
            .output()
            .expect("Couldn't run git status");
        
        (String::from_utf8_lossy(&child.stdout)).lines()
            .filter(|l| l.chars().nth(0).unwrap() == ' ')
            .map(|p| PathBuf::from(&p[3..])).collect()
    }

    pub fn last_commit_msg(&self) -> String {
        let last_commit = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("--no-pager")
            .arg("log")
            .arg("-1")
            .arg("--oneline")
            .arg("--pretty=%B")
            .output()
            .expect("Couldn't get last commit");
        String::from_utf8(last_commit.stdout).unwrap()
    }

    pub fn branch_name(&self) -> String {
        let branch_name = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("branch")
            .arg("--show-current")
            .output()
            .expect("Couldn't get branch name");
        String::from_utf8(branch_name.stdout).unwrap()
    }

    pub fn stage_file(&self, path: &PathBuf) {
        Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("add")
            .arg(path.to_str().unwrap())
            .output()
            .expect("Couldn't stage file");
    }

    pub fn unstage_file(&self, path: &PathBuf) {
        Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("reset")
            .arg("--")
            .arg(path.to_str().unwrap())
            .output()
            .expect("Couldn't stage file");
    }

    pub fn commit(&self, args: Vec<String>, msg: String) {
        Command::new("git")
            .args([vec![String::from("-C"), self.work_dir.to_str().unwrap().to_string(), String::from("commit"), String::from("-m"), msg], args.clone()].concat())
            .output()
            .expect("Couldn't commit");
    }

    pub fn push(&self) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(self.work_dir.to_str().unwrap())
            .arg("push")
            .output()
            .expect("Couldn't commit");
        let err = String::from_utf8(output.stderr).unwrap();
        match err.as_str() {
            "" => String::from("Push Successful!"),
            _  => String::from("Push Failed"),
        }
    }

}
