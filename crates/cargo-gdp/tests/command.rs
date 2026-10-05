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
        "// π keeps editor byte offsets distinct from columns\n#[gp::proof] pub struct P<'a>; impl<'a> P<'a> { pub fn issue_unchecked() {} }\n",
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
    let output = invoke(&[
        "lint",
        "-p",
        "bad",
        "--offline",
        "--message-format=cargo-json",
    ]);
    assert!(!output.status.success());
    let editor: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(editor[0]["reason"], "compiler-message");
    assert!(editor[0]["package_id"].as_str().unwrap().contains("bad"));
    assert_eq!(editor[0]["target"]["name"], "bad");
    assert!(
        editor[0]["target"]["src_path"]
            .as_str()
            .unwrap()
            .ends_with("custom/entry.rs")
    );
    assert_eq!(editor[0]["message"]["code"]["code"], "gdp::exported_issuer");
    let span = &editor[0]["message"]["spans"][0];
    let source = fs::read_to_string(root.join("bad/custom/elsewhere.rs")).unwrap();
    let start = span["byte_start"].as_u64().unwrap() as usize;
    let end = span["byte_end"].as_u64().unwrap() as usize;
    assert_eq!(&source[start..end], "i");
    assert_eq!(span["line_start"], 2);
    assert_eq!(span["is_primary"], true);
    assert_eq!(editor.last().unwrap()["reason"], "build-finished");
    assert_eq!(editor.last().unwrap()["success"], false);
    let output = invoke(&[
        "lint",
        "-p",
        "absent",
        "--offline",
        "--message-format=cargo-json",
    ]);
    assert!(!output.status.success());
    let finished: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(finished["reason"], "build-finished");
    assert_eq!(finished["success"], false);
    assert!(
        !invoke(&["doctor", "--message-format=cargo-json"])
            .status
            .success()
    );
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
    let output = invoke(&[
        "check",
        "-p",
        "clean",
        "--offline",
        "--message-format",
        "cargo-json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let editor: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(
        editor
            .iter()
            .filter(|line| line["reason"] == "build-finished")
            .count(),
        1
    );
    assert_eq!(editor.last().unwrap()["reason"], "build-finished");
    assert_eq!(editor.last().unwrap()["success"], true);
    // Exercise a real expanded proof whose handwritten Clone compiles, but
    // violates the evidence convention. Editor status must reflect GDP failure
    // even though Cargo's compiler stage succeeds.
    let library = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    fs::write(root.join("bad/Cargo.toml"), format!(
        "[package]\nname=\"bad\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[lib]\npath=\"custom/entry.rs\"\n[dependencies]\ngp={{package=\"ghostproof\",path={library:?},features=[\"macros\"]}}\n"
    )).unwrap();
    fs::write(root.join("bad/custom/elsewhere.rs"),
        "\u{feff}#[gp::proof] pub struct P<'a>; impl Clone for P<'_> { fn clone(&self) -> Self { Self { __gdp_brand: core::marker::PhantomData } } }\n"
    ).unwrap();
    let output = invoke(&[
        "check",
        "-p",
        "bad",
        "--offline",
        "--message-format=cargo-json",
    ]);
    assert!(!output.status.success());
    let editor: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(editor.iter().any(|r| r["reason"] == "compiler-message"
        && r["message"]["code"]["code"] == "gdp::duplicated_evidence"));
    let finding = editor
        .iter()
        .find(|r| r["message"]["code"]["code"] == "gdp::duplicated_evidence")
        .unwrap();
    let span = &finding["message"]["spans"][0];
    let source = fs::read_to_string(root.join("bad/custom/elsewhere.rs")).unwrap();
    assert_eq!(
        &source[span["byte_start"].as_u64().unwrap() as usize
            ..span["byte_end"].as_u64().unwrap() as usize],
        "P"
    );
    assert_eq!(editor[editor.len() - 2]["reason"], "gdp-summary");
    assert_eq!(editor[editor.len() - 2]["stage"], "syntax");
    assert_eq!(
        editor
            .iter()
            .filter(|r| r["reason"] == "build-finished")
            .count(),
        1
    );
    assert_eq!(editor.last().unwrap()["success"], false);
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
    let output = invoke(&[
        "check",
        "-p",
        "clean",
        "--offline",
        "--message-format=cargo-json",
    ]);
    assert!(!output.status.success());
    let editor: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(
        editor
            .iter()
            .any(|record| record["reason"] == "compiler-message")
    );
    assert_eq!(
        editor
            .iter()
            .filter(|line| line["reason"] == "build-finished")
            .count(),
        1
    );
    assert_eq!(editor.last().unwrap()["success"], false);
    fs::remove_dir_all(root).unwrap();
}
