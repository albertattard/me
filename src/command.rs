use std::fmt::{Debug, Display, Formatter};

use crate::command::ExecutionMode::{Default, DelayBetweenCommands, Interactive};
use regex::Regex;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub(crate) enum ExecutionMode {
    Default,
    DelayBetweenCommands(u32),
    Interactive,
}

#[derive(Debug)]
pub(crate) struct Options<'a> {
    content: &'a str,
    execute_from: Option<&'a str>,
    execute_until: Option<&'a str>,
    skip_commands: Option<&'a Regex>,
    execution_mode: ExecutionMode,
}

impl<'a> Options<'a> {
    pub(crate) fn new(content: &'a str) -> Self {
        Options {
            content,
            execute_from: None,
            execute_until: None,
            skip_commands: None,
            execution_mode: Default,
        }
    }

    pub(crate) fn with_execute_from(mut self, execute_from: Option<&'a str>) -> Self {
        self.execute_from = execute_from;
        self
    }

    pub(crate) fn with_execute_until(mut self, execute_until: Option<&'a str>) -> Self {
        self.execute_until = execute_until;
        self
    }

    pub(crate) fn with_skip_commands(mut self, skip_commands: Option<&'a Regex>) -> Self {
        self.skip_commands = skip_commands;
        self
    }

    pub(crate) fn with_execution_mode(mut self, execution_mode: ExecutionMode) -> Self {
        self.execution_mode = execution_mode;
        self
    }

    pub(crate) fn build(&'a self) -> Commands<'a> {
        Commands::parse(self).expect("Failed to parse the MARKDOWN file")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParserError {
    message: String,
}

impl ParserError {
    fn new(message: String) -> Self {
        ParserError { message }
    }

    fn err<R>(message: String) -> Result<R, ParserError> {
        Err(Self::new(message))
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParserError {}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Command<'a> {
    command: Vec<&'a str>,
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Commands<'a> {
    /* TODO: Consider switching to a VecDeque given that we pop elements from the front when iterating. */
    commands: Vec<Command<'a>>,
    execution_mode: ExecutionMode,
}

impl<'a> Commands<'a> {
    fn parse(options: &'a Options<'a>) -> Result<Self, ParserError> {
        let mut commands = vec![];
        let mut buffer_command = vec![];

        let mut within_command_block = false;
        let mut execute_from_found = false;
        let mut execute_until_found = false;
        let execute_from = options.execute_from;

        for line in options.content.lines() {
            let trimmed_start_line = line.trim_start();

            if trimmed_start_line.eq("```shell") {
                within_command_block = true;
                continue;
            }

            if trimmed_start_line.eq("```") {
                within_command_block = false;
                continue;
            }

            if let Some(from_line) = execute_from {
                if !execute_from_found && trimmed_start_line.eq_ignore_ascii_case(from_line) {
                    execute_from_found = true;
                } else if !execute_from_found {
                    continue;
                }
            }

            if within_command_block {
                let mut command_line = line.trim_start();
                if command_line.starts_with("$ ") {
                    command_line = command_line[2..].trim_start();
                }

                if command_line.ends_with('\\') {
                    command_line = command_line[..command_line.len() - 1].trim_end();
                    buffer_command.push(command_line);
                    continue;
                }

                buffer_command.push(command_line);

                /* Check if the command needs to be skipped and clear the buffer if so */
                if let Some(regex) = &options.skip_commands {
                    if regex.is_match(&buffer_command.join(" ")) {
                        buffer_command.clear();
                        continue;
                    }
                }

                commands.push(Command {
                    command: buffer_command,
                });
                buffer_command = vec![];
            }

            if options
                .execute_until
                .map_or(false, |m| trimmed_start_line.eq_ignore_ascii_case(m))
            {
                execute_until_found = true;
                break;
            }
        }

        if let Some(from_line) = execute_from {
            if !execute_from_found {
                return ParserError::err(format!(
                    "No line matched the execute from: '{}'",
                    from_line
                ));
            }
        }

        if let Some(until_line) = options.execute_until {
            if !execute_until_found {
                return if let Some(from_line) = execute_from {
                    ParserError::err(format!(
                        "No line matched the execute until: '{}' after the execute from: '{}'",
                        until_line, from_line
                    ))
                } else {
                    ParserError::err(format!(
                        "No line matched the execute until: '{}'",
                        until_line
                    ))
                };
            }
        }

        Ok(Commands {
            commands,
            execution_mode: options.execution_mode,
        })
    }

    pub(crate) fn as_shell_script(&self) -> String {
        let mut buffer_command = String::new();
        buffer_command.push_str(
            r#"#!/bin/sh

# Generated by the MARKDOWN executor
# This file is automatically deleted once the execution completes

set -e

"#,
        );

        match self.execution_mode {
            Default => {
                for command in &self.commands {
                    buffer_command.push_str(format!("{}\n", command).as_str());
                }
            }

            DelayBetweenCommands(delay_in_millis) => {
                let mut commands = self.commands.iter();

                if let Some(first_command) = commands.next() {
                    buffer_command.push_str(format!("{}\n", first_command).as_str());

                    for command in commands {
                        buffer_command.push_str(format!("sleep {}\n", delay_in_millis).as_str());
                        buffer_command.push_str(format!("{}\n", command).as_str());
                    }
                }
            }

            Interactive => {
                buffer_command.push_str(r#"# When set to true, it will execute the remaining commands without interaction
EXECUTE_ALL=false

# Confirms before executing each command.  The command can be skipped and the script exited.
interactive() {
  COLOR_OFF='\033[0m'
  CURSOR_CARET='\033[0;94m'
  COMMAND='\033[0;92m'
  MENU='\033[0;02m'

  if [ "${EXECUTE_ALL}" != true ]; then
    echo "${MENU}--------------------------------------------------${COLOR_OFF}"
    echo "${CURSOR_CARET}>${COLOR_OFF} ${COMMAND}${*}${COLOR_OFF}"
    echo "${MENU}--------------------------------------------------"
    read -r -p "Press enter to execute,
 A to execute all the remaining commands,
 S to skip and
 X to exit " input
    echo "--------------------------------------------------${COLOR_OFF}"

    case ${input} in
      [sS] ) return;;
      [xX] ) exit 0;;
      [aA] ) EXECUTE_ALL=true;
        ;;
      * )
        ;;
    esac
  fi

  # Execute the command
  "$@";
}

"#);
                for command in &self.commands {
                    buffer_command.push_str(format!("interactive {}\n", command).as_str());
                }
            }
        }

        buffer_command
    }
}

impl Display for Commands<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_empty();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_without_commands() {
        let content = r#"# README

No commands here!!
"#;

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_empty();
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_one_single_line_command() {
        let content = r#"# README

Before command

```shell
$ ls -la
```

After command
"#;

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["ls -la"], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_single_line_command() {
        let content = r#"# README

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

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(
            vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""],
            Default,
        );
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_different_indentation() {
        let content = r#"# README

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

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(
            vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""],
            Default,
        );
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_one_multi_line_command() {
        let content = r#"# README

```shell
$ java \
  -jar target/app.jar
```
"#;

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = Ok(Commands {
            commands: vec![Command {
                command: vec!["java", "-jar target/app.jar"],
            }],
            execution_mode: Default,
        });
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_single_line_commands() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(
            vec!["echo \"Line 1\"", "echo \"Line 2\"", "echo \"Line 3\""],
            Default,
        );
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_with_multiple_multi_line_commands() {
        let content = r#"# README

```shell
$ echo "Before"
$ java \
  -jar target/app-1.jar
$ java \
  -jar target/app-2.jar
$ echo "After"
```
"#;

        let options = Options::new(content);
        let parsed = Commands::parse(&options);
        let expected = Ok(Commands {
            commands: vec![
                Command {
                    command: vec!["echo \"Before\""],
                },
                Command {
                    command: vec!["java", "-jar target/app-1.jar"],
                },
                Command {
                    command: vec!["java", "-jar target/app-2.jar"],
                },
                Command {
                    command: vec!["echo \"After\""],
                },
            ],
            execution_mode: Default,
        });
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let options = Options::new(content).with_execute_from(Some("$ echo \"Line 2\""));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Line 2\"", "echo \"Line 3\""], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from_when_no_lines_match() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let from_line = "$ echo \"Line x\"";
        let options = Options::new(content).with_execute_from(Some(from_line));
        let parsed = Commands::parse(&options);
        let expected =
            ParserError::err(format!("No line matched the execute from: '{}'", from_line));
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_until() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let options = Options::new(content).with_execute_until(Some("$ echo \"Line 2\""));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Line 1\"", "echo \"Line 2\""], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_until_when_no_lines_match() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let until_line = "$ echo \"Line x\"";
        let options = Options::new(content).with_execute_until(Some(until_line));
        let parsed = Commands::parse(&options);
        let expected = ParserError::err(format!(
            "No line matched the execute until: '{}'",
            until_line
        ));
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from_and_until() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
$ echo "Line 4"
```
"#;

        let options = Options::new(content)
            .with_execute_from(Some("$ echo \"Line 2\""))
            .with_execute_until(Some("$ echo \"Line 3\""));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Line 2\"", "echo \"Line 3\""], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from_and_until_same_line() {
        let content = r#"# README

```shell
$ echo "Line 1"
```
"#;

        let options = Options::new(content)
            .with_execute_from(Some("$ echo \"Line 1\""))
            .with_execute_until(Some("$ echo \"Line 1\""));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Line 1\""], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from_and_until_when_no_lines_match_until() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
$ echo "Line 4"
```
"#;

        let from_line = "$ echo \"Line 2\"";
        let until_line = "$ echo \"Line 1\"";
        let options = Options::new(content)
            .with_execute_from(Some(from_line))
            .with_execute_until(Some(until_line));
        let parsed = Commands::parse(&options);
        let expected = ParserError::err(format!(
            "No line matched the execute until: '{}' after the execute from: '{}'",
            until_line, from_line
        ));
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_execute_from_and_until_when_until_also_exists_before_from() {
        let content = r#"# README

```shell
$ echo "Line 2"
$ echo "Line 1"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let options = Options::new(content)
            .with_execute_from(Some("$ echo \"Line 1\""))
            .with_execute_until(Some("$ echo \"Line 2\""));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Line 1\"", "echo \"Line 2\""], Default);
        assert_eq!(expected, parsed);
    }

    #[test]
    fn parse_content_skip_commands() {
        let content = r#"# README

```shell
$ echo "Line 1"
$ echo "Hello there"
$ echo "Line 2"
$ echo "Line 3"
```
"#;

        let skip_commands = Regex::new(r"Line \d").expect("Invalid skip commands regex");
        let options = Options::new(content).with_skip_commands(Some(&skip_commands));
        let parsed = Commands::parse(&options);
        let expected = ok_of_strs(vec!["echo \"Hello there\""], Default);
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
        let commands = of_strs(vec!["ls -la"], Default);
        let formatted = format!("{}", commands);
        let expected = r#"ls -la
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_multiple_single_line_command() {
        let commands = of_strs(
            vec!["echo \"Hello\"", "ls -la", "echo \"Goodbye\""],
            Default,
        );
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
                command: vec!["java", "-jar target/app.jar"],
            }],
            execution_mode: Default,
        };
        let formatted = format!("{}", commands);
        let expected = r#"java \
 -jar target/app.jar
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_multiple_single_line_commands() {
        let commands = of_strs(
            vec!["echo \"Line 1\"", "echo \"Line 2\"", "echo \"Line 3\""],
            Default,
        );
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
                    command: vec!["echo \"Before\""],
                },
                Command {
                    command: vec!["java", "-jar target/app-1.jar"],
                },
                Command {
                    command: vec!["java", "-jar target/app-2.jar"],
                },
                Command {
                    command: vec!["echo \"After\""],
                },
            ],
            execution_mode: Default,
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
    fn format_as_shell_script_with_default_execution() {
        let commands = of_strs(
            vec![
                "echo \"Before\"",
                "java -jar target/app-1.jar",
                "java -jar target/app-2.jar",
                "echo \"After\"",
            ],
            Default,
        );
        let formatted = commands.as_shell_script();
        let expected = r#"#!/bin/sh

# Generated by the MARKDOWN executor
# This file is automatically deleted once the execution completes

set -e

echo "Before"
java -jar target/app-1.jar
java -jar target/app-2.jar
echo "After"
"#;
        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_as_shell_script_with_delay_between_commands_execution() {
        let commands = of_strs(
            vec!["echo \"Line 1\"", "echo \"Line 2\"", "echo \"Line 3\""],
            DelayBetweenCommands(100),
        );

        let formatted = commands.as_shell_script();
        let expected = r#"#!/bin/sh

# Generated by the MARKDOWN executor
# This file is automatically deleted once the execution completes

set -e

echo "Line 1"
sleep 100
echo "Line 2"
sleep 100
echo "Line 3"
"#;

        assert_eq!(expected, formatted);
    }

    #[test]
    fn format_as_shell_script_with_interactive_execution() {
        let commands = of_strs(
            vec!["echo \"Line 1\"", "echo \"Line 2\"", "echo \"Line 3\""],
            Interactive,
        );

        let formatted = commands.as_shell_script();
        let expected = r#"#!/bin/sh

# Generated by the MARKDOWN executor
# This file is automatically deleted once the execution completes

set -e

# When set to true, it will execute the remaining commands without interaction
EXECUTE_ALL=false

# Confirms before executing each command.  The command can be skipped and the script exited.
interactive() {
  COLOR_OFF='\033[0m'
  CURSOR_CARET='\033[0;94m'
  COMMAND='\033[0;92m'
  MENU='\033[0;02m'

  if [ "${EXECUTE_ALL}" != true ]; then
    echo "${MENU}--------------------------------------------------${COLOR_OFF}"
    echo "${CURSOR_CARET}>${COLOR_OFF} ${COMMAND}${*}${COLOR_OFF}"
    echo "${MENU}--------------------------------------------------"
    read -r -p "Press enter to execute,
 A to execute all the remaining commands,
 S to skip and
 X to exit " input
    echo "--------------------------------------------------${COLOR_OFF}"

    case ${input} in
      [sS] ) return;;
      [xX] ) exit 0;;
      [aA] ) EXECUTE_ALL=true;
        ;;
      * )
        ;;
    esac
  fi

  # Execute the command
  "$@";
}

interactive echo "Line 1"
interactive echo "Line 2"
interactive echo "Line 3"
"#;

        assert_eq!(expected, formatted);
    }

    fn ok_empty() -> Result<Commands<'static>, ParserError> {
        Ok(empty())
    }

    fn ok_of_strs(
        commands: Vec<&str>,
        execution_mode: ExecutionMode,
    ) -> Result<Commands<'_>, ParserError> {
        Ok(of_strs(commands, execution_mode))
    }

    fn empty() -> Commands<'static> {
        Commands {
            commands: vec![],
            execution_mode: Default,
        }
    }

    fn of_strs(commands: Vec<&str>, execution_mode: ExecutionMode) -> Commands<'_> {
        let commands = commands
            .iter()
            .map(|command| Command {
                command: vec![command],
            })
            .collect();
        Commands {
            commands,
            execution_mode,
        }
    }
}
