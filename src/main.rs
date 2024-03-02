use crate::cla::Args;
use crate::command::{Commands, Options};
use crate::shell::ShellScript;

mod cla;
mod command;
mod shell;

fn main() {
    let args = Args::create();
    let options = Options::new(args.read_file());
    let shell_script = Commands::parse(&options).as_shell_script();
    ShellScript::new(shell_script).run();
}
