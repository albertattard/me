use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    command: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn parse(content: &str) -> io::Result<Self> {
        let mut commands = vec![];
        let mut buffer_command = vec![];
        let mut within_command_block = false;
        for line in content.lines() {
            if line.trim().eq("```shell") {
                within_command_block = true;
                continue;
            }

            if line.trim().eq("```") {
                within_command_block = false;
                continue;
            }

            if within_command_block {
                let mut command_line = line.to_string();
                if command_line.starts_with("$ ") {
                    command_line = command_line[2..].to_string();
                }

                if command_line.ends_with("\\") {
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

        Ok(Commands { commands })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_content() {
        let content = "";
        let parsed = Commands::parse(content).unwrap();
        let expected = empty();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_without_commands() {
        let content = "";
        let parsed = Commands::parse(content).unwrap();
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

        let parsed = Commands::parse(content).unwrap();
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

        let parsed = Commands::parse(content).unwrap();
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

        let parsed = Commands::parse(content).unwrap();
        let expected = Commands {
            commands: vec![Command {
                command: vec!["java".to_string(), "-jar target/app.jar".to_string()],
            }],
        };
        assert_eq!(expected, parsed);
    }

    pub fn empty() -> Commands {
        Commands { commands: vec![] }
    }

    pub fn of_strs(commands: Vec<&str>) -> Commands {
        let commands = commands
            .iter()
            .map(|command| Command {
                command: vec![command.to_string()],
            })
            .collect();
        Commands { commands }
    }
}
