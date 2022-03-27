mod cli;

use clap::StructOpt;
use cli::CliInput;
use simple_logger::SimpleLogger;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use git2::{ErrorCode, Repository};
use globset::{Glob, GlobMatcher};
use rayon::prelude::*;

use crate::cli::CliOptions;

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
    pub fn from_json(json_value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(config_obj) = json_value {
            let items = config_obj
                .into_iter()
                .map(|(glob_pat, commands)| {
                    let path_matcher = globset::Glob::new(&glob_pat).unwrap().compile_matcher();
                    let commands = match commands {
                        serde_json::Value::String(command) => {
                            vec![command]
                        }
                        serde_json::Value::Array(_) => {
                            let commands: Vec<String> = serde_json::from_value(commands).unwrap();
                            commands
                        }
                        _ => unreachable!(),
                    };
                    RslintStagedConfigItem {
                        path_matcher,
                        commands,
                    }
                })
                .collect();
            RslintStagedConfig { items }
        } else {
            panic!("Unvalid config")
        }
    }
}

fn get_rslint_staged_config<T: AsRef<Path>>(cwd: T) -> RslintStagedConfig {
    let cwd = cwd.as_ref();
    let package_json_file_path = cwd.join("package.json");
    let maybe_lint_staged_json_value = if package_json_file_path.exists() {
        let mut pkg_json_value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(package_json_file_path).unwrap())
                .unwrap();
        pkg_json_value
            .get_mut("lint-staged")
            .map(|item| item.take())
    } else {
        None
    };
    let lint_staged_json_value = maybe_lint_staged_json_value.unwrap_or_else(|| {
        let lintstagedrc_json_file_path = cwd.join(".lintstagedrc.json");
        serde_json::from_str(&std::fs::read_to_string(lintstagedrc_json_file_path).unwrap())
            .unwrap()
    });
    RslintStagedConfig::from_json(lint_staged_json_value)
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
    cli_options: CliOptions,
    repo: Repo,
}

impl RslintStaged {
    pub fn exec(&self) {
        let staged_files = self.repo.staged_files();
        let cwd = &self.cli_options.cwd;
        log::debug!("staged_files {:?}", staged_files);
        self.config.items.par_iter().for_each(|config_item| {
            let filterd = staged_files
                .iter()
                .filter(|path| config_item.path_matcher.is_match(path))
                .collect::<Vec<_>>();
            config_item.commands.iter().for_each(|command| {
                let parsed = command
                    .split_ascii_whitespace()
                    .into_iter()
                    .collect::<Vec<_>>();
                Command::new(parsed[0])
                    .current_dir(cwd)
                    .args(&parsed[1..])
                    .args(&filterd)
                    .spawn()
                    .unwrap();
            });
        });
    }
}

fn main() {
    let cli: CliOptions = CliInput::parse().into();
    if cli.debug {
        SimpleLogger::new().init().unwrap();
    }
    log::debug!("CliOptions: {:?}", cli);
    let cwd = &cli.cwd;
    let config = get_rslint_staged_config(&cwd);
    let repo = Repo {
        raw: Repository::open(cwd).expect("Not a git dir"),
        root: cwd.to_owned(),
    };
    log::debug!("cwd: {:?}", cwd);
    let rslint_staged = RslintStaged {
        repo,
        cli_options: cli,
        config,
    };
    rslint_staged.exec();
}
