#![warn(missing_debug_implementations, rust_2018_idioms)]

use crate::cla::Args;
use crate::command::Options;
use crate::shell::ShellScript;

mod cla;
mod command;
mod shell;

fn main() {
    let args = Args::create();

    for markdown in args.files() {
        let shell_script = Options::new(&markdown.read())
            .with_execute_from(args.execute_from())
            .with_execute_until(args.execute_until())
            .with_skip_commands(args.skip_commands())
            .with_execution_mode(args.execution_mode())
            .with_no_colour(args.no_colour())
            .with_prefix_commands(args.prefix_commands_with())
            .build()
            .as_shell_script();

        ShellScript::new(&markdown.parent_dir(), &shell_script).run();
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use assert_cmd::Command;

    const COMMAND_COLOUR: &str = "\u{1b}[0;02m";
    const NO_COLOUR: &str = "\u{1b}[0m";

    #[test]
    fn run_with_no_args() {
        let dir = "./target/fixtures/run_with_no_args";
        remove_fixtures(dir);
        new_fixture(
            &format!("{}/README.md", dir),
            r#"# README Fixture

Print `Hello world!!`

```shell
$ echo 'Hello world!!'
```
"#,
        );

        Command::cargo_bin("../release/me")
            .expect("Failed to create test command")
            .current_dir(dir)
            .assert()
            .stdout(format!(
                r#"{COMMAND_COLOUR}---{NO_COLOUR}
{COMMAND_COLOUR}$ echo 'Hello world!!'{NO_COLOUR}
Hello world!!
"#
            ))
            .success();
    }

    #[test]
    fn run_with_some_args() {
        let dir = "./target/fixtures/run_with_some_args";
        remove_fixtures(dir);
        new_fixture(
            &format!("{}/README.md", dir),
            r#"# README Fixture

Print some messages

```shell
$ echo 'Hello 1!!'
$ echo 'Hello 2!!'
$ echo 'Line 1!!'
$ echo 'Line 2!!'
$ echo 'Line 3!!'
$ echo 'Line 4!!'
$ echo 'Hello 3!!'
$ echo 'Hello 4!!'
```
"#,
        );

        Command::cargo_bin("../release/me")
            .expect("Failed to create test command")
            .current_dir(dir)
            .args([
                "--execute-from",
                "$ echo 'Hello 2!!'",
                "--execute-until",
                "$ echo 'Hello 3!!'",
                "--skip-commands",
                "Line \\d+",
            ])
            .assert()
            .stdout(format!(
                r#"{COMMAND_COLOUR}---{NO_COLOUR}
{COMMAND_COLOUR}$ echo 'Hello 2!!'{NO_COLOUR}
Hello 2!!
{COMMAND_COLOUR}---{NO_COLOUR}
{COMMAND_COLOUR}$ echo 'Hello 3!!'{NO_COLOUR}
Hello 3!!
"#
            ))
            .success();
    }

    #[test]
    fn run_with_recursive_args() {
        let dir = "./target/fixtures/run_with_recursive_args";
        remove_fixtures(dir);
        new_fixture(
            &format!("{}/README.md", dir),
            r#"# README Fixture
```shell
$ echo 'Level 1'
```
"#,
        );

        new_fixture(
            &format!("{}/a/README.md", dir),
            r#"# README Fixture
```shell
$ echo 'Level 2'
```
"#,
        );

        new_fixture(
            &format!("{}/a/b/README.md", dir),
            r#"# README Fixture
```shell
$ echo 'Level 3'
```
"#,
        );

        Command::cargo_bin("../release/me")
            .expect("Failed to create test command")
            .current_dir(dir)
            .args(["--recursive"])
            .assert()
            .stdout(format!(
                r#"{COMMAND_COLOUR}---{NO_COLOUR}
{COMMAND_COLOUR}$ echo 'Level 1'{NO_COLOUR}
Level 1
{COMMAND_COLOUR}---{NO_COLOUR}
{COMMAND_COLOUR}$ echo 'Level 2'{NO_COLOUR}
Level 2
"#
            ))
            .success();
    }

    fn new_fixture(fixture_path: &str, content: &str) {
        let path = Path::new(fixture_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create the missing parent directories");
        }

        File::create(path)
            .expect("Failed to create test fixture")
            .write_all(content.as_bytes())
            .expect("Failed to write content to test fixture");
    }

    fn remove_fixtures<P>(directory: P)
    where
        P: AsRef<Path>,
    {
        if directory.as_ref().exists() {
            fs::remove_dir_all(directory).expect("Failed to remove all fixtures");
        }
    }
}
