use std::fs::File;
use std::io::Write;
use std::os::unix::prelude::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

pub(crate) struct ShellScript {
    path: PathBuf,
}

impl ShellScript {
    pub(crate) fn new(directory: &Path, commands: &str) -> Self {
        let script_path = Self::create_file_path(directory);

        Self::create_shell_script(&script_path)
            .write_all(commands.as_bytes())
            .expect("Failed to create shell script");

        ShellScript { path: script_path }
    }

    pub(crate) fn run(&self) {
        Command::new("/bin/sh")
            .current_dir(&self.current_dir())
            .args(["-c", &self.path_as_str()])
            .spawn()
            .expect("Failed to execute process")
            .wait()
            .expect("Failed to finish process");
    }

    fn path_as_str(&self) -> String {
        fs::canonicalize(&self.path)
            .expect("Failed to canonicalize path")
            .as_path()
            .as_os_str()
            .to_str()
            .expect("failed to convert path")
            .to_string()
    }

    fn current_dir(&self) -> PathBuf {
        fs::canonicalize(&self.path)
            .expect("Failed to canonicalize path")
            .parent()
            .map(|path| path.to_path_buf())
            .unwrap_or_else(|| env::current_dir().expect("Failed to fetch the current directory"))
    }

    fn create_file_path(directory: &Path) -> PathBuf {
        directory.join(format!("commands-{}.sh", Self::millis_since_epoch()))
    }

    fn millis_since_epoch() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }

    fn create_shell_script(path: &Path) -> File {
        let shell_script = File::create(path).expect("Failed to create shell script");
        Self::make_shell_script_executable(&shell_script);
        shell_script
    }

    fn make_shell_script_executable(shell_script: &File) {
        let metadata = shell_script
            .metadata()
            .expect("Failed to get the script metadata");

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);

        shell_script
            .set_permissions(permissions)
            .expect("Failed to set the script permissions");
    }
}

impl Drop for ShellScript {
    fn drop(&mut self) {
        if fs::remove_file(&self.path).is_err() {
            eprintln!("Failed to delete the auto generated shell script");
        }
    }
}
