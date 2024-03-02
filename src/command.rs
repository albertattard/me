use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Options {
    content: String,
}

impl Options {
    pub(crate) fn new(content: String) -> Self {
        Options { content }
    }

    pub(crate) fn build(self) -> Commands {
        Commands::parse(&self)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Command {
    command: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    fn parse(options: &Options) -> Self {
        let mut commands = vec![];
        let mut buffer_command = vec![];
        let mut within_command_block = false;
        for line in options.content.lines() {
            if line.trim().eq("```shell") {
                within_command_block = true;
                continue;
            }

            if line.trim().eq("```") {
                within_command_block = false;
                continue;
            }

            if within_command_block {
                let mut command_line = line.trim_start().to_string();
                if command_line.starts_with("$ ") {
                    command_line = command_line[2..].to_string();
                }

                if command_line.ends_with('\\') {
                    command_line = command_line[..command_line.len() - 1]
                        .trim_end()
                        .to_string();
                    buffer_command.push(command_line);
                    continue;
                }

                buffer_command.push(command_line.trim_start().to_string());
                commands.push(Command {
                    command: buffer_command.clone(),
                });
                buffer_command.clear();
            }
        }

        Commands { commands }
    }

    pub(crate) fn as_shell_script(&self) -> String {
        let mut buffer_command = String::new();
        buffer_command.push_str(
            r#"#!/bin/sh

set -e

"#,
        );
        buffer_command.push_str(&self.to_string());
        buffer_command
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut command = self.command.iter();

        if let Some(first_line) = command.next() {
            write!(f, "{}", first_line)?;

            for line in command {
                writeln!(f, " \\")?;
                write!(f, " {}", line)?;
            }
        }

        Ok(())
    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for command in &self.commands {
            writeln!(f, "{}", command)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_content() {
        let content = "";
        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = empty();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_without_commands() {
        let content = r#"
# README

No commands here!!
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = empty();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_one_single_line_command() {
        let content = r#"
# README

Before command

```shell
$ ls -la
```

After command
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = of_strs(vec!["ls -la"]);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_single_line_command() {
        let content = r#"
# README

```shell
$ echo "Hello"
```

```shell
$ ls -la
```

```shell
$ echo "Goodbye"
```

"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = of_strs(vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""]);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_different_indentation() {
        let content = r#"
# README

```shell
$ echo "Hello"
```

- `ls` command

  ```shell
  $ ls -la
  ```

  1. `echo` command

     ```shell
     $ echo "Goodbye"
     ```
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = of_strs(vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""]);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_one_multi_line_command() {
        let content = r#"
# README
```shell
$ java \
  -jar target/app.jar
```
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = Commands {
            commands: vec![Command {
                command: vec!["java".to_string(), "-jar target/app.jar".to_string()],
            }],
        };
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_single_line_commands() {
        let content = r#"
# README
```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = of_strs(vec![
            "echo \"Line 1\"",
            "echo \"Line 2\"",
            "echo \"Line 3\"",
        ]);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_multi_line_commands() {
        let content = r#"
# README
```shell
$ echo "Before"
$ java \
  -jar target/app-1.jar
$ java \
  -jar target/app-2.jar
$ echo "After"
```
"#;

        let options = Options::new(content.to_string());
        let parsed = Commands::parse(&options);
        let expected = Commands {
            commands: vec![
                Command {
                    command: vec!["echo \"Before\"".to_string()],
                },
                Command {
                    command: vec!["java".to_string(), "-jar target/app-1.jar".to_string()],
                },
                Command {
                    command: vec!["java".to_string(), "-jar target/app-2.jar".to_string()],
                },
                Command {
                    command: vec!["echo \"After\"".to_string()],
                },
            ],
        };
        assert_eq!(expected, parsed);
    }

    #[test]
    fn format_empty_command() {
        let commands = empty();
        let formatted = format!("{}", commands);
        let expected = "";
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_one_single_line_command() {
        let commands = of_strs(vec!["ls -la"]);
        let formatted = format!("{}", commands);
        let expected = r#"ls -la
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_multiple_single_line_command() {
        let commands = of_strs(vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""]);
        let formatted = format!("{}", commands);
        let expected = r#"echo "Hello"
ls -la
echo "Goodbye"
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_one_multi_line_command() {
        let commands = Commands {
            commands: vec![Command {
                command: vec!["java".to_string(), "-jar target/app.jar".to_string()],
            }],
        };
        let formatted = format!("{}", commands);
        let expected = r#"java \
 -jar target/app.jar
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_multiple_single_line_commands() {
        let commands = of_strs(vec![
            "echo \"Line 1\"",
            "echo \"Line 2\"",
            "echo \"Line 3\"",
        ]);
        let formatted = format!("{}", commands);
        let expected = r#"echo "Line 1"
echo "Line 2"
echo "Line 3"
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_multiple_multi_line_commands() {
        let commands = Commands {
            commands: vec![
                Command {
                    command: vec!["echo \"Before\"".to_string()],
                },
                Command {
                    command: vec!["java".to_string(), "-jar target/app-1.jar".to_string()],
                },
                Command {
                    command: vec!["java".to_string(), "-jar target/app-2.jar".to_string()],
                },
                Command {
                    command: vec!["echo \"After\"".to_string()],
                },
            ],
        };
        let formatted = format!("{}", commands);
        let expected = r#"echo "Before"
java \
 -jar target/app-1.jar
java \
 -jar target/app-2.jar
echo "After"
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_as_shell_script() {
        let commands = of_strs(vec![
            "echo \"Before\"",
            "java -jar target/app-1.jar",
            "java -jar target/app-2.jar",
            "echo \"After\"",
        ]);
        let formatted = commands.as_shell_script();
        let expected = r#"#!/bin/sh

set -e

echo "Before"
java -jar target/app-1.jar
java -jar target/app-2.jar
echo "After"
"#;
        assert_eq!(expected, formatted);
    }

    fn empty() -> Commands {
        Commands { commands: vec![] }
    }

    fn of_strs(commands: Vec<&str>) -> Commands {
        let commands = commands
            .iter()
            .map(|command| Command {
                command: vec![command.to_string()],
            })
            .collect();
        Commands { commands }
    }
}
