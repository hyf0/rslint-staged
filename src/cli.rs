use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct CliInput {
    /// Allow empty commits when tasks revert all staged changes (default: false)
    #[clap(short, long)]
    pub allow_empty: bool,
    /// disable lint-staged’s own console output (default: false)
    #[clap(short, long)]
    pub quiet: bool,
    /// pass relative filepaths to tasks (default: false)
    #[clap(short, long)]
    pub relative: bool,
    /// To run concurrently or not (default: true)
    #[clap(long)]
    pub concurrent: bool,
    /// path to configuration file
    #[clap(short, long, parse(from_os_str), value_name = "path")]
    pub config: Option<PathBuf>,
    /// run all tasks in specific directory, instead of the current
    #[clap(long, parse(from_os_str), value_name = "path")]
    pub cwd: Option<PathBuf>,
    /// skip parsing of tasks for better shell support (default: false)
    #[clap(long)]
    pub shell: bool,
    /// Turn debugging information on
    #[clap(short, long)]
    pub debug: bool,
    /// show task output even when tasks succeed; by default only failed output is shown
    #[clap(short, long)]
    pub verbose: bool,
    /// By default a backup stash will be created before running the tasks, and all task modifications will be reverted in case of an error. This option will disable creating the stash, and instead leave all modifications in the index when aborting the commit.
    #[clap(long)]
    pub no_stash: bool,
}

impl From<CliInput> for CliOptions {
    fn from(input: CliInput) -> Self {
        Self {
            allow_empty: input.allow_empty,
            quiet: input.quiet,
            relative: input.relative,
            concurrent: input.concurrent,
            config: input.config,
            cwd: input
                .cwd
                .unwrap_or_else(|| std::env::current_dir().unwrap()),
            shell: input.shell,

            debug: input.debug,
            verbose: input.verbose,
            no_stash: input.no_stash,
        }
    }
}

#[derive(Debug)]
pub struct CliOptions {
    pub allow_empty: bool,
    pub quiet: bool,
    pub relative: bool,
    pub concurrent: bool,
    pub config: Option<PathBuf>,
    pub cwd: PathBuf,
    pub shell: bool,
    pub debug: bool,
    pub verbose: bool,
    pub no_stash: bool,
}
