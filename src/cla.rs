use clap::Parser;
use regex::Regex;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Name of the MARKDOWN file to parse
    #[arg(short, long, default_value = "README.md", display_order = 1)]
    file_name: String,

    /// Executes from the given command, or line within the file.  When provided, anything before
    /// this line is ignored.  The matching command is not ignored.  If no matching lines are found,
    /// then the program will panic.  If more than one line matches, the program will start from
    /// the first matching line.
    #[arg(short, long, display_order = 2)]
    execute_from: Option<String>,

    /// Executes until the given command, or line within the file.  When provided, anything after
    /// this line is ignored.  The matching command is not ignored.  If no matching lines are found,
    /// then the program will panic.  If more than one line matches, the program will stop at the
    /// first matching line.
    #[arg(short, long, display_order = 3)]
    execute_until: Option<String>,

    /// Skips all commands that match the provided regular expression.  Nothing happens if the given
    /// regular expression does not match any commands.
    #[arg(short, long, display_order = 4)]
    skip_commands: Option<Regex>,
}

impl Args {
    pub(crate) fn create() -> Self {
        Args::parse()
    }

    pub(crate) fn execute_from(&self) -> Option<String> {
        self.execute_from.clone()
    }

    pub(crate) fn execute_until(&self) -> Option<String> {
        self.execute_until.clone()
    }

    pub(crate) fn skip_commands(&self) -> Option<Regex> {
        self.skip_commands.clone()
    }

    pub(crate) fn read_file(&self) -> String {
        read_to_string(self.file_path())
            .unwrap_or_else(|_| panic!("Failed to read MARKDOWN file: {}", self.file_name))
    }

    fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
}
