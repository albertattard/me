use crate::cla::Args;
use std::io;
use std::path::Path;

mod cla;
mod command;

fn main() {
    let args = Args::create();
    println!("Executing {:?}", args.file_path());
}

fn read_file(file_path: &Path) -> io::Result<String> {
    std::fs::read_to_string(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_file_that_exists() {
        let file_path = Path::new("target/fixtures/README.md");
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        std::fs::write(file_path, "# README\n\nThis is a README file\n").unwrap();

        let content = read_file(file_path).unwrap();
        assert_eq!(content, "# README\n\nThis is a README file\n");
    }
}
