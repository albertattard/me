use crate::cla::Args;
use crate::command::Options;
use crate::shell::ShellScript;

mod cla;
mod command;
mod shell;

fn main() {
    let args = Args::create();
    let shell_script = Options::new(args.read_file())
        .with_execute_from(args.execute_from())
        .build()
        .as_shell_script();
    ShellScript::new(shell_script).run();
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use assert_cmd::Command;

    #[test]
    fn run_with_no_args() {
        let dir = "./target/fixtures/1";
        let path = &format!("{}/README.md", dir);
        let content = r#"
# README

Print `Hello world!!`

```shell
$ echo 'Hello world!!'
```
"#;

        Fixture::new(path, content).consume(|| {
            Command::cargo_bin("../release/me")
                .expect("Failed to create test command")
                .current_dir(dir)
                .assert()
                .stdout("Hello world!!\n")
                .success();
        })
    }

    #[test]
    fn run_with_execute_from() {
        let dir = "./target/fixtures/2";
        let path = &format!("{}/README.md", dir);
        let content = r#"
# README

Print `Hello world!!`

```shell
$ echo 'Hello world!!'
```

Print `Hello there!!`

```shell
$ echo 'Hello there!!'
```
"#;

        Fixture::new(path, content).consume(|| {
            Command::cargo_bin("../release/me")
                .expect("Failed to create test command")
                .current_dir(dir)
                .args(["--execute-from", "Print `Hello there!!`"])
                .assert()
                .stdout("Hello there!!\n")
                .success();
        });
    }

    struct Fixture {
        path: String,
    }

    impl Fixture {
        fn new(path: &str, content: &str) -> Self {
            let p = Path::new(path);

            if let Some(parent) = p.parent() {
                fs::create_dir_all(parent)
                    .expect("Failed to create the missing parent directories");
            }

            File::create(p)
                .expect("Failed to create test fixture")
                .write(content.as_bytes())
                .expect("Failed to write content to test fixture");

            Fixture {
                path: path.to_string(),
            }
        }

        fn consume<F>(self, f: F)
        where
            F: FnOnce(),
        {
            f()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if fs::remove_file(&self.path).is_err() {
                eprintln!("Failed to delete the fixture");
            }
        }
    }
}
