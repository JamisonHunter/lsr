use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

fn temp_dir(name: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = env::temp_dir().join(format!(
        "lsr-cli-test-{}-{}-{}",
        name,
        std::process::id(),
        counter
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn command_in(dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lsr"));
    command
        .current_dir(dir)
        .env_remove("LSR_CONFIG")
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR")
        .env_remove("CLICOLOR_FORCE");
    command
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn rejects_extra_arguments() {
    let dir = temp_dir("extra-arguments");

    let output = command_in(&dir)
        .args(["-a", "unexpected"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unexpected argument"));
    assert!(stderr.contains("--help"));

    cleanup(&dir);
}

#[test]
fn help_and_version_exit_successfully() {
    let dir = temp_dir("help-version");

    let help = command_in(&dir).arg("--help").output().unwrap();
    assert!(help.status.success());
    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(help_stdout.contains("Usage:"));
    assert!(help_stdout.contains("Exit status:"));

    let version = command_in(&dir).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        env!("CARGO_PKG_VERSION")
    );

    cleanup(&dir);
}

#[test]
fn complete_scan_exits_zero_and_piped_output_has_no_ansi() {
    let dir = temp_dir("no-ansi");
    fs::write(dir.join("file.txt"), b"hello").unwrap();

    let output = command_in(&dir).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("Current directory size:"));
    assert!(!stdout.contains('\x1b'));

    cleanup(&dir);
}

#[test]
fn no_color_environment_is_honored() {
    let dir = temp_dir("no-color");
    fs::write(dir.join("file.txt"), b"hello").unwrap();

    let output = command_in(&dir).env("NO_COLOR", "1").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(!stdout.contains('\x1b'));

    cleanup(&dir);
}

#[test]
fn dash_a_controls_dotfile_display() {
    let dir = temp_dir("dotfiles");
    fs::write(dir.join(".hidden"), b"hidden").unwrap();
    fs::write(dir.join("visible"), b"visible").unwrap();

    let default_output = command_in(&dir).output().unwrap();
    let default_stdout = String::from_utf8_lossy(&default_output.stdout);
    assert!(default_stdout.contains("visible"));
    assert!(!default_stdout.contains(".hidden"));

    let all_output = command_in(&dir).arg("-a").output().unwrap();
    let all_stdout = String::from_utf8_lossy(&all_output.stdout);
    assert!(all_stdout.contains("visible"));
    assert!(all_stdout.contains(".hidden"));

    cleanup(&dir);
}

#[cfg(unix)]
#[test]
fn escapes_control_characters_in_filenames() {
    let dir = temp_dir("control-names");
    fs::write(dir.join("bad\nname"), b"x").unwrap();
    fs::write(dir.join("escape\x1b[31mred"), b"y").unwrap();

    let output = command_in(&dir).arg("-a").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains("bad\\nname"));
    assert!(stdout.contains("escape\\u{1b}[31mred"));
    assert!(!stdout.contains('\x1b'));
    assert!(!stdout.contains("bad\nname"));

    cleanup(&dir);
}

#[cfg(unix)]
#[test]
fn incomplete_scan_exits_one_and_marks_output() {
    use std::os::unix::fs::PermissionsExt;

    let dir = temp_dir("incomplete");
    let locked = dir.join("locked");
    fs::create_dir(&locked).unwrap();
    fs::write(dir.join("visible.txt"), b"x").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

    // Permission restrictions are bypassed when running as root and on some
    // filesystems. Skip the test in that case rather than asserting the wrong
    // behavior.
    if fs::read_dir(&locked).is_ok() {
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        cleanup(&dir);
        return;
    }

    let output = command_in(&dir).output().unwrap();

    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    cleanup(&dir);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr.contains("cannot access"));
    assert!(stdout.contains("(incomplete)"));
}
