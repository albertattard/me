use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::{env, fs};

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

    /// Skips all commands that match the provided regular expression.  Nothing happens if the given
    /// regular expression does not match any commands.
    #[arg(short, long)]
    skip_commands: Option<Regex>,

    /// Searches for MARKDOWN files, named README.md or the provided file name, in the
    /// subdirectories and execute each MARKDOWN file from the directory it was found.
    #[arg(short, long, num_args = 0..=1, value_name = "DEPTH", default_missing_value = "2")]
    recursive: Option<usize>,
}

impl Args {
    pub(crate) fn create() -> Self {
        Args::parse()
    }

    pub(crate) fn skip_commands(&self) -> Option<&Regex> {
        self.skip_commands.as_ref()
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
