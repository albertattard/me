use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::{env, fs};

use crate::command::ExecutionMode;
use clap::Parser;
use regex::Regex;
use walkdir::WalkDir;

/// A simple application that parses markdown files and executes the shell code blocks.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Name of the MARKDOWN file to parse
    #[arg(short, long, default_value = "README.md")]
    file_name: String,

    /// Executes from the given command, or line within the file.  When provided, anything before
    /// this line is ignored.  The matching command is not ignored.  If no matching lines are found,
    /// then the program will panic.  If more than one line matches, the program will start from
    /// the first matching line.
    #[arg(short = 'b', long, num_args = 0..=1, default_missing_value = "---EXECUTE-FROM-HERE---")]
    execute_from: Option<String>,

    /// Executes until the given command, or line within the file.  When provided, anything after
    /// this line is ignored.  The matching command is not ignored.  If no matching lines are found,
    /// then the program will panic.  If more than one line matches, the program will stop at the
    /// first matching line.
    #[arg(short = 'e', long, num_args = 0..=1, default_missing_value = "---EXECUTE-UNTIL-HERE---")]
    execute_until: Option<String>,

    /// Skips all commands that match the provided regular expression.  Nothing happens if the given
    /// regular expression does not match any commands.
    #[arg(short, long)]
    skip_commands: Option<Regex>,

    /// Introduce a delay (in milliseconds) between each command
    #[arg(short, long, conflicts_with = "interactive")]
    delay_between_commands: Option<u32>,

    /// Prompts for a confirmation before executing each command, ideal when debugging a problem
    /// with a MARKDOWN file.
    #[arg(
        short,
        long,
        conflicts_with = "delay_between_commands",
        num_args = 0,
        required = false
    )]
    interactive: bool,

    /// Searches for MARKDOWN files, named README.md or the provided file name, in the
    /// subdirectories and execute each MARKDOWN file from the directory it was found.
    #[arg(short, long, num_args = 0..=1, value_name = "DEPTH", default_missing_value = "2")]
    recursive: Option<usize>,

    /// Prefix all commands with the given command.  For example, say you need to time all commands
    /// using the `time` command, then you can use this option to prefix all commands found within
    /// the MARKDOWN file with `time`.
    #[arg(short, long, value_name = "COMMAND")]
    prefix_commands_with: Option<String>,
}

impl Args {
    pub(crate) fn create() -> Self {
        Args::parse()
    }

    pub(crate) fn execute_from(&self) -> Option<&str> {
        self.execute_from.as_deref()
    }

    pub(crate) fn execute_until(&self) -> Option<&str> {
        self.execute_until.as_deref()
    }

    pub(crate) fn skip_commands(&self) -> Option<&Regex> {
        self.skip_commands.as_ref()
    }

    pub(crate) fn execution_mode(&self) -> ExecutionMode {
        if self.interactive {
            ExecutionMode::Interactive
        } else if let Some(delay_in_millis) = self.delay_between_commands {
            ExecutionMode::DelayBetweenCommands(delay_in_millis)
        } else {
            ExecutionMode::Default
        }
    }

    pub(crate) fn prefix_commands_with(&self) -> Option<&str> {
        self.prefix_commands_with.as_deref()
    }

    pub(crate) fn files(&self) -> Vec<MarkdownFile> {
        self.recursive
            .map(|max_depth| Self::find_markdown_files(max_depth, &self.file_name))
            .unwrap_or_else(|| vec![MarkdownFile::new(self.file_path())])
    }

    fn find_markdown_files(max_depth: usize, file_name: &str) -> Vec<MarkdownFile> {
        WalkDir::new(env::current_dir().expect("Failed to get the current working directory"))
            .max_depth(max_depth)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok()) // Convert iterator of `Result<DirEntry, Error>` to iterator of `DirEntry`
            .filter(|e| e.file_type().is_file()) // Filter to only consider files
            .filter(|e| e.file_name() == file_name) // Filter for files named "MARKDOWN.md"
            .map(|e| e.into_path()) // Convert DirEntry to PathBuf
            .map(MarkdownFile::new)
            .collect()
    }

    fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
}

pub(crate) struct MarkdownFile {
    path: PathBuf,
}

impl MarkdownFile {
    fn new(path: PathBuf) -> Self {
        MarkdownFile { path }
    }

    pub(crate) fn parent_dir(&self) -> PathBuf {
        fs::canonicalize(&self.path)
            .expect("Failed to canonicalize path")
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                env::current_dir().expect("Failed to get the current working directory")
            })
    }

    pub(crate) fn read(&self) -> String {
        read_to_string(&self.path)
            .unwrap_or_else(|_| panic!("Failed to read MARKDOWN file: {}", self.path_as_str()))
    }

    fn path_as_str(&self) -> String {
        fs::canonicalize(&self.path)
            .expect("Failed to canonicalize path")
            .as_os_str()
            .to_str()
            .expect("failed to convert path")
            .to_string()
    }
}

impl Display for MarkdownFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path_as_str())
    }
}
