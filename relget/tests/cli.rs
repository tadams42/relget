//! End-to-end tests driving the built `relget` binary.
//!
//! All tests are network-free: the registry and completion generator are compiled into the
//! binary, and the install test runs `--offline` against an empty `XDG_CACHE_HOME`, which is
//! a soft skip by design.

use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str], cache_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_relget"))
        .args(args)
        .env("XDG_CACHE_HOME", cache_dir)
        .output()
        .expect("failed to run relget")
}

#[test]
fn registry_list_apps_ids_prints_sorted_ids() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(&["registry", "list-apps-ids"], tmp.path());
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    let ids: Vec<&str> = stdout.lines().collect();
    assert!(ids.contains(&"ripgrep"), "expected 'ripgrep' in registry ids");
    assert!(ids.windows(2).all(|w| w[0] <= w[1]), "ids are not sorted");
}

#[test]
fn completions_zsh_emits_script() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(&["completions", "zsh"], tmp.path());
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("relget"), "completion script does not mention relget");
}

#[test]
fn apps_and_configured_set_flags_conflict() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(&["install", "--apps", "ripgrep", "--configured-set", "s"], tmp.path());
    assert_eq!(out.status.code(), Some(2), "expected clap usage-error exit code");
}

#[test]
fn install_unknown_app_fails_with_nonzero_exit() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(&["install", "--apps", "no-such-app-xyz", "--offline"], tmp.path());
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Unknown app"), "stderr: {stderr}");
}

#[test]
fn uninstall_from_empty_prefix_removes_nothing() {
    let cache = tempfile::tempdir().unwrap();
    let prefix = tempfile::tempdir().unwrap();
    let prefix_arg = prefix.path().to_str().unwrap();
    let out = run(
        &["uninstall", "--apps", "ripgrep", "--prefix", prefix_arg],
        cache.path(),
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("No files removed."), "stdout: {stdout}");
}

#[test]
fn offline_install_with_empty_cache_is_a_soft_skip() {
    let cache = tempfile::tempdir().unwrap();
    let prefix = tempfile::tempdir().unwrap();
    let prefix_arg = prefix.path().to_str().unwrap();
    let out = run(
        &[
            "--offline",
            "install",
            "--apps",
            "ripgrep",
            "--prefix",
            prefix_arg,
        ],
        cache.path(),
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !prefix.path().join("bin").exists(),
        "nothing should have been installed"
    );
}
