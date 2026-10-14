//! End-to-end CLI checks that do not need the app (APP-028).

use assert_cmd::Command;

fn cli() -> Command {
    let mut c = Command::cargo_bin("xcreenshot").unwrap();
    // Isolated endpoint and data folder: no running app can answer.
    c.env(
        "XCREENSHOT_IPC_NAME",
        format!("xcreenshot-clitest-{}", std::process::id()),
    )
    .env("XCREENSHOT_DATA_DIR", std::env::temp_dir().join("xs-clitest"))
    .env("XCREENSHOT_APP", "does-not-exist");
    c
}

#[test]
fn no_launch_exits_with_3_and_json_error() {
    let out = cli()
        .args(["app", "ping", "--no-launch", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is only JSON");
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "not_running");
}

#[test]
fn usage_errors_exit_with_2() {
    let out = cli()
        .args(["settings", "open", "--section", "nope"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let out = cli().args(["run", "nope.nope", "--json"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn destructive_commands_need_yes() {
    let out = cli()
        .args(["shortcuts", "reset", "--json", "--no-launch"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn help_lists_parameters_in_both_languages() {
    let out = cli()
        .args(["help", "shortcuts.set", "--lang", "en"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    for p in ["COMMAND", "SHORTCUT", "--scope", "--reassign"] {
        assert!(text.contains(p), "{p} missing in:\n{text}");
    }
    let out = cli().args(["--lang", "es", "--help"]).output().unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("Captura de pantalla"));
}

/// Definition of done: every command is documented in AGENTS.md.
#[test]
fn agents_md_lists_every_command() {
    let agents = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../AGENTS.md")).unwrap();
    for spec in xs_core::catalog::specs() {
        assert!(
            agents.contains(&format!("`{}`", spec.id)),
            "{} missing from AGENTS.md",
            spec.id
        );
    }
}
