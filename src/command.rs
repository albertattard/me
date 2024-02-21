use std::io;
use std::path::Path;

pub struct Command {
    command: Vec<String>,
}

pub struct Commands {
    commands: Vec<Command>,
}

impl Commands {
    pub fn parse(content: &str) -> io::Result<Self> {
        Ok(Commands { commands: vec![] })
    }

    pub fn size(&self) -> usize {
        self.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_file() {
        let content = "";
        let commands = Commands::parse(content).unwrap();
        assert_eq!(commands.size(), 0);
    }
}
