use std::{fs, process::Command};

#[test]
fn command_reports_limits_and_scans_only_selected_packages() {
    let root = std::env::temp_dir().join(format!("cargo-gdp-test-{}", std::process::id()));
    fs::create_dir_all(root.join("clean/src")).unwrap();
    fs::create_dir_all(root.join("bad/src")).unwrap();
    fs::create_dir_all(root.join("bad/custom/policy")).unwrap();
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers=[\"clean\",\"bad\"]\ndefault-members=[\"clean\"]\nresolver=\"3\"\n",
    )
    .unwrap();
    for name in ["clean", "bad"] {
        fs::write(
            root.join(name).join("Cargo.toml"),
            format!("[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2024\"\n"),
        )
        .unwrap();
    }
    fs::write(root.join("clean/src/lib.rs"), "pub fn harmless() {}\n").unwrap();
    // Syntax-only scanning deliberately does not need this unresolved attribute
    // to typecheck. Real protected declarations are separately compiler-tested.
    fs::write(root.join("bad/Cargo.toml"), "[package]\nname=\"bad\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[lib]\npath=\"custom/entry.rs\"\n").unwrap();
    fs::write(root.join("bad/custom/entry.rs"), "mod policy;\n").unwrap();
    fs::write(
        root.join("bad/custom/policy.rs"),
        "#[path=\"elsewhere.rs\"] mod nested;\n",
    )
    .unwrap();
    fs::write(
        root.join("bad/custom/elsewhere.rs"),
        "#[gp::proof] pub struct P<'a>; impl<'a> P<'a> { pub fn issue_unchecked() {} }\n",
    )
    .unwrap();
    let invoke = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_cargo-gdp"))
            .current_dir(&root)
            .args(args)
            .env_remove("CARGO")
            .output()
            .unwrap()
    };
    let output = invoke(&["gdp", "doctor", "--offline"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clean:"));
    assert!(!stdout.contains("bad:"));
    assert!(stdout.contains("not inside stock Clippy"));
    assert!(
        invoke(&["lint", "-p", "clean", "--offline"])
            .status
            .success()
    );
    let output = invoke(&["lint", "--workspace", "--exclude", "clean", "--offline"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("gdp::exported_issuer"));
    let output = invoke(&["lint", "-p", "bad", "--offline", "--message-format=json"]);
    assert!(!output.status.success());
    let lines: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "a custom source must not be scanned twice");
    assert_eq!(lines[0]["reason"], "gdp-diagnostic");
    assert_eq!(lines[0]["code"], "gdp::exported_issuer");
    assert!(lines[0]["file"].as_str().unwrap().ends_with("elsewhere.rs"));
    assert!(lines[0]["column"].as_u64().unwrap() > 0);
    assert_eq!(lines[1]["reason"], "gdp-summary");
    assert_eq!(lines[1]["success"], false);
    assert_eq!(lines[0]["schema_version"], 1);
    assert!(invoke(&["--version"]).status.success());
    let output = invoke(&["check", "-p", "clean", "--offline", "--message-format=json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lines: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.last().unwrap()["reason"], "gdp-summary");
    assert_eq!(lines.last().unwrap()["success"], true);
    let output = invoke(&["doctor", "--offline", "--message-format=json"]);
    assert!(output.status.success());
    let doctor: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(doctor["reason"], "gdp-doctor");
    assert_eq!(doctor["packages"][0]["name"], "clean");
    assert!(!invoke(&["not-a-command"]).status.success());
    fs::write(root.join("clean/src/lib.rs"), "pub fn broken( {}\n").unwrap();
    let output = invoke(&["check", "-p", "clean", "--offline"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error"));
    let output = invoke(&[
        "check",
        "-p",
        "clean",
        "--offline",
        "--message-format=json-render-diagnostics",
    ]);
    assert!(!output.status.success());
    let lines: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.last().unwrap()["reason"], "gdp-summary");
    assert_eq!(lines.last().unwrap()["stage"], "compiler");
    assert_eq!(lines.last().unwrap()["success"], false);
    fs::remove_dir_all(root).unwrap();
}
