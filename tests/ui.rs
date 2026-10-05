//! Test real downstream boundaries without compiler-version-specific snapshots.
#![cfg(feature = "macros")]

use std::{fs, path::Path, process::Command};

#[test]
fn downstream_compile_contracts() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let directory = repository
        .join("target")
        .join(format!("gdp-ui-{}", std::process::id()));
    fs::create_dir_all(directory.join("src/bin/support")).unwrap();
    let manifest = format!(
        "[package]\nname = \"gdp-contract-fixtures\"\nversion = \"0.0.0\"\nedition = \"2024\"\nrust-version = \"1.85\"\n[workspace]\n[dependencies]\ngp = {{ package = \"ghostproof\", path = {:?}, features = [\"macros\"] }}\n",
        repository
    );
    fs::write(directory.join("Cargo.toml"), manifest).unwrap();
    fs::copy(
        repository.join("tests/ui/support/mod.rs"),
        directory.join("src/bin/support/mod.rs"),
    )
    .unwrap();
    let mut cases: Vec<_> = fs::read_dir(repository.join("tests/ui"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    cases.sort();
    // Compile the actual README example rather than a manually maintained copy.
    let readme = fs::read_to_string(repository.join("README.md")).unwrap();
    let sample = readme
        .split("```rust\n")
        .nth(1)
        .unwrap()
        .split("```")
        .next()
        .unwrap();
    let readme_case = directory.join("src/bin/pass_readme.rs");
    fs::write(
        &readme_case,
        format!("extern crate gp as ghostproof;\n{sample}"),
    )
    .unwrap();
    for case in &cases {
        fs::copy(
            case,
            directory.join("src/bin").join(case.file_name().unwrap()),
        )
        .unwrap();
    }
    let mut failures = Vec::new();
    cases.push(readme_case);
    for case in cases {
        let source = fs::read_to_string(&case).unwrap();
        let pattern = source
            .lines()
            .find_map(|line| line.strip_prefix("// error-pattern: "));
        let output = Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet", "--manifest-path"])
            .arg(directory.join("Cargo.toml"))
            .arg("--bin")
            .arg(case.file_stem().unwrap())
            .env("CARGO_TARGET_DIR", directory.join("build"))
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        match pattern {
            Some(pattern)
                if !output.status.success() && pattern.split('|').any(|p| stderr.contains(p)) => {}
            None if output.status.success() => {}
            _ => failures.push(format!(
                "{}: expected {:?}, status {}\n{}",
                case.display(),
                pattern,
                output.status,
                stderr
            )),
        }
    }
    let _ = fs::remove_dir_all(directory);
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
