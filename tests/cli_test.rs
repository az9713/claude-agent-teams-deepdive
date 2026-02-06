use assert_cmd::Command;
use predicates::prelude::*;

fn todos() -> Command {
    Command::cargo_bin("todos").unwrap()
}

#[test]
fn test_help_flag() {
    todos()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A fast, cross-language TODO linter"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("scan"));
}

#[test]
fn test_version_flag() {
    todos()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("todos"));
}

#[test]
fn test_list_on_fixtures() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO"))
        .stdout(predicate::str::contains("FIXME"))
        .stdout(predicate::str::contains("Summary"));
}

#[test]
fn test_scan_alias() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures", "scan"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO"));
}

#[test]
fn test_default_command_is_list() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures"])
        .assert()
        .success()
        .stdout(predicate::str::contains("TODO"));
}

#[test]
fn test_tag_filter() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures", "--tag=FIXME", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FIXME"))
        .stdout(predicate::str::is_match("HACK").unwrap().not());
}

#[test]
fn test_multiple_tag_filter() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures", "--tag=HACK,BUG", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("HACK"))
        .stdout(predicate::str::contains("BUG"));
}

#[test]
fn test_count_format() {
    todos()
        .args(["--color=never", "--path", "tests/fixtures", "--format=count"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+$").unwrap());
}

#[test]
fn test_empty_directory() {
    let dir = tempfile::TempDir::new().unwrap();
    todos()
        .args(["--color=never", "--path", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 TODOs in 0 files"));
}
