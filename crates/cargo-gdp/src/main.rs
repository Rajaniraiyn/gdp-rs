//! Cargo integration without a compiler fork or implicit toolchain changes.
mod analysis;
mod diagnostic;
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
    let forwarded = &args[usize::from(!args.is_empty())..];
    match command.as_str() {
        "--version" | "-V" => {
            println!("cargo-gdp {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        "help" | "--help" | "-h" => {
            println!(
                "cargo gdp check [cargo check options]\ncargo gdp lint [workspace selection options] [--message-format=json]\ncargo gdp doctor [workspace selection options]\n\ncheck runs cargo check, then narrow syntax checks on selected local packages.\nJSON mode emits gdp-diagnostic and gdp-summary objects on stdout.\nNo command modifies manifests or toolchains. Syntax checks do not certify policy truth,\nresolve aliases, expand macros, or inspect every conditional compilation path."
            );
            Ok(0)
        }
        "check" => {
            let status = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
                .arg("check")
                .args(forwarded)
                .status()
                .map_err(|e| e.to_string())?;
            if !status.success() {
                if json_output(forwarded)? {
                    println!(
                        "{}",
                        serde_json::json!({"reason":"gdp-summary", "schema_version":1, "success":false, "stage":"compiler", "diagnostics":0})
                    );
                }
                return Ok(status.code().unwrap_or(1).clamp(1, 255) as u8);
            }
            lint(forwarded)
        }
        "lint" => lint(forwarded),
        "doctor" => {
            let packages = workspace::selected(forwarded)?;
            let rustc = Command::new("rustc")
                .arg("--version")
                .output()
                .map_err(|e| e.to_string())?;
            if !rustc.status.success() {
                return Err(String::from_utf8_lossy(&rustc.stderr).into_owned());
            }
            if json_output(forwarded)? {
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

fn lint(args: &[String]) -> Result<u8, String> {
    let json = json_output(args)?;
    let mut diagnostics = Vec::new();
    for package in workspace::selected(args)? {
        analysis::scan_package(&package, &mut diagnostics)?;
    }
    for diagnostic in &diagnostics {
        if json {
            println!("{}", diagnostic.json());
        } else {
            eprintln!("{diagnostic}");
        }
    }
    if json {
        println!(
            "{}",
            serde_json::json!({"reason":"gdp-summary", "schema_version":1, "success":diagnostics.is_empty(), "stage":"syntax", "diagnostics":diagnostics.len(), "analysis":"syntax", "limits":"No alias resolution, macro expansion, cfg evaluation, or policy verification"})
        );
        return Ok(u8::from(!diagnostics.is_empty()));
    }
    if diagnostics.is_empty() {
        println!(
            "GDP syntax checks passed. This does not certify checks, policy truth, or all API boundaries."
        );
        Ok(0)
    } else {
        Ok(1)
    }
}

fn json_output(args: &[String]) -> Result<bool, String> {
    let mut value = None;
    for (index, arg) in args.iter().enumerate() {
        if arg == "--message-format" {
            value = Some(
                args.get(index + 1)
                    .ok_or("missing value for --message-format")?
                    .as_str(),
            );
        }
        if let Some(format) = arg.strip_prefix("--message-format=") {
            value = Some(format);
        }
    }
    Ok(value.is_some_and(|format| {
        format
            .split(',')
            .any(|part| part == "json" || part.starts_with("json-"))
    }))
}
