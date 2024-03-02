use crate::cla::Args;
use crate::command::Options;
use crate::shell::ShellScript;

mod cla;
mod command;
mod shell;

fn main() {
    let args = Args::create();
    let shell_script = Options::new(args.read_file()).build().as_shell_script();
    ShellScript::new(shell_script).run();
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use assert_cmd::Command;

    #[test]
    fn run_with_no_args() {
        let path = "./target/fixtures/README.md";
        let content = r#"
# README

Hello world!
```shell
$ echo 'Hello world!!'
```
"#;

        write_fixture(path, content);

        Command::cargo_bin("../release/me")
            .expect("Failed to create test command")
            .current_dir("./target/fixtures")
            .assert()
            .stdout("Hello world!!\n")
            .success();
    }

    fn write_fixture(path: &str, content: &str) {
        File::create(path)
            .expect("Failed to create test fixture")
            .write(content.as_bytes())
            .expect("Failed to write content to test fixture");
    }
}
