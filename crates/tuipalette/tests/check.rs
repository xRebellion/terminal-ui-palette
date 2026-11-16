//! The `--check` path end to end: the binary, a real file, an exit code.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tuipalette"))
}

fn tempfile(name: &str, body: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("tuipalette-cli-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, body).unwrap();
    path
}

const GOOD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tuipalette-core/tests/fixtures/launcher.ron"
);
const GOOD_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tuipalette-core/tests/fixtures/launcher.json"
);

#[test]
fn a_valid_spec_exits_zero() {
    let out = bin().args(["--check", "--spec", GOOD]).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("4 panes"), "{stdout}");
    assert!(stdout.contains("3 bindings"), "{stdout}");
}

#[test]
fn the_json_adapter_reads_the_same_document() {
    let out = bin().args(["--check", "--spec", GOOD_JSON]).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn an_undeclared_pane_exits_one_and_names_it() {
    let path = tempfile(
        "ghost.ron",
        r#"(window: (title: "x"), layout: (pane: "ghost"), panes: [])"#,
    );
    let out = bin().args(["--check", "--spec"]).arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("ghost"), "{stderr}");
    assert!(stderr.contains("layout.pane"), "{stderr}");
}

#[test]
fn a_syntax_error_exits_one_with_a_position() {
    let path = tempfile("broken.ron", r#"(window: (title: "x""#);
    let out = bin().args(["--check", "--spec"]).arg(&path).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("line 1"), "{:?}", out);
}

#[test]
fn a_misused_invocation_exits_two() {
    let out = bin().args(["--check"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--spec"));
}
