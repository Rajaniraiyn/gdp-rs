//! Cargo integration without a compiler fork or implicit toolchain changes.
mod analysis;
mod diagnostic;
mod output;
mod rules;
mod workspace;

use std::{
    env,
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("cargo-gdp: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<u8, String> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    if args.first().is_some_and(|arg| arg == "gdp") {
        args.remove(0);
    }
    let command = args
        .first()
        .map(String::as_str)
        .unwrap_or("help")
        .to_owned();
    let (format, forwarded) = output::parse(&args[usize::from(!args.is_empty())..])?;
    let forwarded = forwarded.as_slice();
    let result = execute(&command, forwarded, format);
    if result.is_err() {
        output::finished(format, false);
    }
    result
}

fn execute(command: &str, forwarded: &[String], format: output::Format) -> Result<u8, String> {
    match command {
        "--version" | "-V" => {
            println!("cargo-gdp {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        "help" | "--help" | "-h" => {
            println!(
                "cargo gdp check [cargo check options]\ncargo gdp lint [workspace selection options] [--message-format=json]\ncargo gdp doctor [workspace selection options]\n\ncheck runs cargo check, then narrow syntax checks on selected local packages.\nJSON mode emits gdp-diagnostic and gdp-summary objects on stdout.\nUse --message-format=cargo-json with check or lint for Cargo/rustc editor diagnostics.\nNo command modifies manifests or toolchains. Syntax checks do not certify policy truth,\nresolve aliases, expand macros, or inspect every conditional compilation path."
            );
            Ok(0)
        }
        "check" => {
            let status = output::check(forwarded, format)?;
            if !status.success() {
                if format.is_json() {
                    println!(
                        "{}",
                        serde_json::json!({"reason":"gdp-summary", "schema_version":1, "success":false, "stage":"compiler", "diagnostics":0})
                    );
                }
                output::finished(format, false);
                return Ok(status.code().unwrap_or(1).clamp(1, 255) as u8);
            }
            lint(forwarded, format)
        }
        "lint" => lint(forwarded, format),
        "doctor" => {
            if format == output::Format::CargoJson {
                return Err("cargo-json is for check or lint; use json for doctor".into());
            }
            let packages = workspace::selected(forwarded)?;
            let rustc = Command::new("rustc")
                .arg("--version")
                .output()
                .map_err(|e| e.to_string())?;
            if !rustc.status.success() {
                return Err(String::from_utf8_lossy(&rustc.stderr).into_owned());
            }
            if format.is_json() {
                let selected: Vec<_> = packages.iter().map(|package| serde_json::json!({"name":package.name,"directory":package.directory,"sources":package.sources})).collect();
                println!(
                    "{}",
                    serde_json::json!({"reason":"gdp-doctor", "schema_version":1, "version":env!("CARGO_PKG_VERSION"), "rustc":String::from_utf8_lossy(&rustc.stdout).trim(), "packages":selected, "analysis":"syntax", "limits":"No alias resolution, macro expansion, cfg evaluation, or policy verification"})
                );
                return Ok(0);
            }
            println!("{}", String::from_utf8_lossy(&rustc.stdout).trim());
            for package in packages {
                println!("{}: {}", package.name, package.directory.display());
            }
            println!(
                "Core enforcement: Rust privacy, ownership, and invariant brands.\nAnalysis: source syntax only; macros and type aliases are not resolved.\nCustom rules run here, not inside stock Clippy.\nExternal state freshness and trusted checker correctness require application review."
            );
            Ok(0)
        }
        _ => Err(format!("unknown command {command:?}; run cargo gdp --help")),
    }
}

fn lint(args: &[String], format: output::Format) -> Result<u8, String> {
    let mut count = 0;
    for package in workspace::selected(args)? {
        let mut diagnostics = Vec::new();
        analysis::scan_package(&package, &mut diagnostics)?;
        count += diagnostics.len();
        for diagnostic in &diagnostics {
            match format {
                output::Format::CargoJson => {
                    println!("{}", output::diagnostic(diagnostic, &package)?)
                }
                output::Format::Json => println!("{}", diagnostic.json()),
                output::Format::Human => eprintln!("{diagnostic}"),
            }
        }
    }
    let success = count == 0;
    if format.is_json() {
        println!(
            "{}",
            serde_json::json!({"reason":"gdp-summary", "schema_version":1, "success":success, "stage":"syntax", "diagnostics":count, "analysis":"syntax", "limits":"No alias resolution, macro expansion, cfg evaluation, or policy verification"})
        );
    } else if success {
        println!(
            "GDP syntax checks passed. This does not certify checks, policy truth, or all API boundaries."
        );
    }
    output::finished(format, success);
    Ok(u8::from(!success))
}
