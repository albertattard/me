use std::fs::read_to_string;

use crate::cla::Args;
use crate::command::Commands;
use crate::shell::ShellScript;

mod cla;
mod command;
mod shell;

fn main() {
    let args = Args::create();

    let content = read_to_string(args.file_path()).expect("failed to read MARKDOWN file");
    let commands = Commands::parse(&content).expect("failed to parse MARKDOWN file");

    ShellScript::new(commands.as_shell_script()).run();
}
