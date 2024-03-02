use clap::Parser;
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

    pub fn file_path(&self) -> PathBuf {
        PathBuf::from(&self.file_name)
    }
}
