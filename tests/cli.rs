use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn kyle() -> Command {
    cargo_bin_cmd!("kyle")
}

fn version_pattern() -> predicates::str::RegexPredicate {
    predicate::str::is_match(r"kyle v\d+\.\d+\.\d+").unwrap()
}

// =============================================================================
// Help & Version
// =============================================================================

#[test]
fn help_command() {
    kyle()
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("kyle - task runner"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn help_flag() {
    kyle()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("kyle - task runner"));
}

#[test]
fn help_short_flag() {
    kyle()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("kyle - task runner"));
}

#[test]
fn version_command() {
    kyle()
        .arg("version")
        .assert()
        .success()
        .stdout(version_pattern());
}

#[test]
fn version_flag() {
    kyle()
        .arg("--version")
        .assert()
        .success()
        .stdout(version_pattern());
}

#[test]
fn version_short_flag() {
    kyle()
        .arg("-v")
        .assert()
        .success()
        .stdout(version_pattern());
}

// =============================================================================
// Task Listing
// =============================================================================

#[test]
fn list_tasks_with_kylefile() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: yaml
name: test-project

tasks:
  build:
    desc: Build the project
    run: echo building
  test:
    desc: Run tests
    run: echo testing
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Available tasks:"))
        .stdout(predicate::str::contains("build"))
        .stdout(predicate::str::contains("test"));
}

#[test]
fn no_kylefile_error() {
    let temp = TempDir::new().unwrap();

    kyle()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("No Kylefile found"))
        .stderr(predicate::str::contains("kyle init"));
}

// =============================================================================
// Task Execution
// =============================================================================

#[test]
fn run_simple_task() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: yaml
name: test

tasks:
  hello:
    run: echo "hello world"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn run_task_with_deps() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: yaml
name: test

tasks:
  first:
    run: echo "first"
  second:
    run: echo "second"
    deps: [first]
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("second")
        .assert()
        .success()
        .stdout(predicate::str::contains("first"))
        .stdout(predicate::str::contains("second"));
}

#[test]
fn task_not_found() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: yaml
name: test

tasks:
  build:
    run: echo "build"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("task not found: nonexistent"));
}

#[test]
fn arg_passthrough() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: toml
name = "test"

[tasks.echo]
run = "echo args:"
"#,
    )
    .unwrap();

    // Test args without --
    kyle()
        .current_dir(temp.path())
        .arg("echo")
        .arg("--flag1")
        .arg("--flag2")
        .arg("value")
        .assert()
        .success()
        .stdout(predicate::str::contains("args: --flag1 --flag2 value"));
}

#[test]
fn arg_passthrough_with_separator() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: toml
name = "test"

[tasks.echo]
run = "echo args:"
"#,
    )
    .unwrap();

    // Test args with -- separator (should work the same)
    kyle()
        .current_dir(temp.path())
        .arg("echo")
        .arg("--")
        .arg("--release")
        .arg("-v")
        .assert()
        .success()
        .stdout(predicate::str::contains("args: --release -v"));
}

// =============================================================================
// Config Commands
// =============================================================================

#[test]
fn config_list() {
    kyle()
        .arg("config")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("default_format"));
}

#[test]
fn config_get() {
    kyle()
        .arg("config")
        .arg("get")
        .arg("default_format")
        .assert()
        .success();
}

#[test]
fn config_get_unknown_key() {
    kyle()
        .arg("config")
        .arg("get")
        .arg("unknown_key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown config key"));
}

#[test]
fn config_path() {
    kyle()
        .arg("config")
        .arg("path")
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

// =============================================================================
// Init Command
// =============================================================================

#[test]
fn init_with_name_toml() {
    let temp = TempDir::new().unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("init")
        .arg("myproject")
        .arg("--toml")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Kylefile"));

    let kylefile = temp.path().join("Kylefile");
    assert!(kylefile.exists());

    let content = fs::read_to_string(&kylefile).unwrap();
    assert!(content.contains("# kyle: toml"));
    assert!(content.contains("version = "));
    assert!(content.contains("name = \"myproject\""));
}

#[test]
fn init_with_name_yaml() {
    let temp = TempDir::new().unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("init")
        .arg("myproject")
        .arg("--yaml")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created Kylefile"));

    let kylefile = temp.path().join("Kylefile");
    assert!(kylefile.exists());

    let content = fs::read_to_string(&kylefile).unwrap();
    assert!(content.contains("# kyle: yaml"));
    assert!(content.contains("version:"));
    assert!(content.contains("name: myproject"));
}

// =============================================================================
// Format Detection
// =============================================================================

#[test]
fn yaml_extension() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile.yaml");
    fs::write(
        &kylefile,
        r#"name: test
tasks:
  hello:
    run: echo "yaml"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("yaml"));
}

#[test]
fn toml_extension() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile.toml");
    fs::write(
        &kylefile,
        r#"name = "test"

[tasks.hello]
run = "echo toml"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("toml"));
}

#[test]
fn header_format_detection_yaml() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: yaml
name: test
tasks:
  hello:
    run: echo "detected yaml"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("detected yaml"));
}

#[test]
fn header_format_detection_toml() {
    let temp = TempDir::new().unwrap();
    let kylefile = temp.path().join("Kylefile");
    fs::write(
        &kylefile,
        r#"# kyle: toml
name = "test"

[tasks.hello]
run = "echo detected-toml"
"#,
    )
    .unwrap();

    kyle()
        .current_dir(temp.path())
        .arg("hello")
        .assert()
        .success()
        .stdout(predicate::str::contains("detected-toml"));
}
