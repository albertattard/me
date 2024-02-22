use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ShellScript {
    path: PathBuf,
}

impl ShellScript {
    pub fn new(commands: String) -> Self {
        let path = Self::create_file_path();

        Self::create_shell_script(&path)
            .write_all(commands.as_bytes())
            .expect("Failed to create shell script");

        ShellScript { path }
    }

    pub fn run(&self) {
        Command::new("/bin/sh")
            .args(["-c", self.path_as_str()])
            .spawn()
            .expect("Failed to execute process")
            .wait()
            .expect("Failed to finish process");
    }

    fn path_as_str(&self) -> &str {
        self.path
            .as_os_str()
            .to_str()
            .expect("failed to convert path")
    }

    fn create_file_path() -> PathBuf {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        PathBuf::from(format!("./commands-{}.sh", since_the_epoch.as_millis()))
    }

    fn create_shell_script(path: &PathBuf) -> File {
        let shell_script = File::create(path).expect("Failed to create shell script");
        let metadata = shell_script
            .metadata()
            .expect("Failed to get the script metadata");

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        shell_script
            .set_permissions(permissions)
            .expect("Failed to set the script permissions");

        shell_script
    }
}

impl Drop for ShellScript {
    fn drop(&mut self) {
        if fs::remove_file(&self.path).is_err() {
            eprintln!("Failed to delete the shell script");
        }
    }
}
