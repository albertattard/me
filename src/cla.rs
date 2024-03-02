use clap::Parser;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    /// Name of the MARKDOWN file to parse
    #[arg(short, long, default_value = "README.md")]
    file_name: String,
}

impl Args {
    pub(crate) fn create() -> Self {
        Args::parse()
    }

    fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }

    pub(crate) fn read_file(&self) -> String {
        read_to_string(self.file_path())
            .unwrap_or_else(|_| panic!("Failed to read MARKDOWN file: {}", self.file_name))
    }
}
