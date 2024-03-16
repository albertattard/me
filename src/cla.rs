use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Name of the MARKDOWN file to parse
    #[arg(short, long, default_value = "README.md")]
    file_name: String,

    /// Executes from the given command, or line within the file.  When provided, anything before
    /// this line is ignored.  The matching command is not ignored.  If no matching lines are found,
    /// then the program will panic.  If more than one line matches, the program will start from
    /// the first matching line.
    #[arg(short, long)]
    execute_from: Option<String>,
}

impl Args {
    pub(crate) fn create() -> Self {
        Args::parse()
    }

    pub(crate) fn execute_from(&self) -> Option<String> {
        self.execute_from.clone()
    }

    pub(crate) fn read_file(&self) -> String {
        read_to_string(self.file_path())
            .unwrap_or_else(|_| panic!("Failed to read MARKDOWN file: {}", self.file_name))
    }

    fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
}
