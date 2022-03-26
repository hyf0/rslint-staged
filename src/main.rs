use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{ErrorCode, Repository};
use globset::{Glob, GlobMatcher};
use rayon::prelude::*;

#[derive(Debug)]
struct RslintStagedConfig {
    pub items: Vec<RslintStagedConfigItem>,
}

#[derive(Debug)]
struct RslintStagedConfigItem {
    pub path_matcher: GlobMatcher,
    pub commands: Vec<String>,
}

impl RslintStagedConfig {
    pub fn from_lintstagedrc_json() {}
    pub fn from_json(json_value: serde_json::Value) -> Self {
        let scripts: HashMap<String, Vec<String>> = serde_json::from_value(json_value).unwrap();
        let items = scripts
            .into_iter()
            .map(|(glob_pat, commands)| RslintStagedConfigItem {
                path_matcher: globset::Glob::new(&glob_pat).unwrap().compile_matcher(),
                commands,
            })
            .collect();
        RslintStagedConfig { items }
    }
}

fn get_rslint_staged_config<T: AsRef<Path>>(project_root_dir: T) -> RslintStagedConfig {
    let lintstagedrc_json_file_path = project_root_dir.as_ref().join(".lintstagedrc.json");
    RslintStagedConfig::from_json(
        serde_json::from_str(&std::fs::read_to_string(lintstagedrc_json_file_path).unwrap())
            .unwrap(),
    )
}

struct Repo {
    raw: Repository,
    pub root: PathBuf,
}
impl Repo {
    pub fn staged_files(&self) -> Vec<PathBuf> {
        let repo = &self.raw;

        let head_tree = repo.head().unwrap().peel_to_tree().unwrap();
        let diff = repo
            .diff_tree_to_index(Some(&head_tree), None, None)
            .unwrap();
        let mut staged_files = diff
            .deltas()
            .flat_map(|delta| vec![delta.old_file().path(), delta.new_file().path()])
            .filter_map(std::convert::identity)
            .map(|path| self.root.join(path))
            .collect::<Vec<_>>();
        staged_files.dedup();
        staged_files
    }
}

impl std::fmt::Debug for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repo").finish()
    }
}

#[derive(Debug)]
struct RslintStaged {
    config: RslintStagedConfig,
    repo: Repo,
    root: PathBuf,
}

impl RslintStaged {
    pub fn new(work_dir: impl AsRef<Path>) -> Self {
        let config = get_rslint_staged_config(work_dir.as_ref());
        let repo = Repo {
            raw: Repository::open(work_dir.as_ref()).expect("Not a git dir"),
            root: work_dir.as_ref().to_owned(),
        };

        Self {
            config,
            repo,
            root: work_dir.as_ref().to_owned(),
        }
    }

    pub fn exec(&self) {
        let staged_files = self.repo.staged_files();
        let cwd = &self.root;
        println!("staged_files {:?}", staged_files);
        self.config.items.par_iter().for_each(|config_item| {
            let filterd = staged_files
                .iter()
                .filter(|path| config_item.path_matcher.is_match(path))
                .collect::<Vec<_>>();
            config_item.commands.iter().for_each(|command| {
                println!("run command {:?}", command);
                Command::new(command)
                    .current_dir(cwd)
                    .args(&filterd)
                    .spawn()
                    .unwrap();
            });
        });
    }
}

fn main() {
    println!("Hello, world!");
    let cwd = std::env::current_dir().unwrap();
    let rslint_staged = RslintStaged::new(cwd);
    rslint_staged.exec();
}
