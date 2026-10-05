//! Optional Cargo/rustc-compatible records for editor check commands.
use crate::{diagnostic::Diagnostic, workspace::Package};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::{BufRead, BufReader},
    process::{Command, ExitStatus, Stdio},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Human,
    Json,
    CargoJson,
}

impl Format {
    pub fn is_json(self) -> bool {
        self != Self::Human
    }
}

/// Keep Cargo options intact except for the tool's editor-format extension.
pub fn parse(args: &[String]) -> Result<(Format, Vec<String>), String> {
    let mut forwarded = Vec::new();
    let mut format = Format::Human;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        let value = if arg == "--message-format" {
            index += 1;
            Some(
                args.get(index)
                    .ok_or("missing value for --message-format")?
                    .as_str(),
            )
        } else {
            arg.strip_prefix("--message-format=")
        };
        if let Some(value) = value {
            format = if value == "cargo-json" {
                Format::CargoJson
            } else if value
                .split(',')
                .any(|p| p == "json" || p.starts_with("json-"))
            {
                Format::Json
            } else {
                Format::Human
            };
            forwarded.push(format!(
                "--message-format={}",
                if value == "cargo-json" { "json" } else { value }
            ));
        } else {
            forwarded.push(arg.clone());
        }
        index += 1;
    }
    Ok((format, forwarded))
}

/// In editor mode, defer build-finished until GDP checks have also completed.
pub fn check(args: &[String], format: Format) -> Result<ExitStatus, String> {
    let mut command = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    command.arg("check").args(args);
    if format != Format::CargoJson {
        return command.status().map_err(|e| e.to_string());
    }
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let stdout = child.stdout.take().ok_or("Cargo stdout was not piped")?;
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|e| e.to_string())?;
        let finished = serde_json::from_str::<Value>(&line)
            .is_ok_and(|record| record["reason"] == "build-finished");
        if !finished {
            println!("{line}");
        }
    }
    child.wait().map_err(|e| e.to_string())
}

pub fn finished(format: Format, success: bool) {
    if format == Format::CargoJson {
        println!("{}", json!({"reason":"build-finished", "success":success}));
    }
}

pub fn diagnostic(diagnostic: &Diagnostic, package: &Package) -> Result<Value, String> {
    // Declared source traversal retains its originating target. Leftover source
    // files use the package's first product target solely for editor association.
    let target = package
        .targets
        .iter()
        .find(|target| {
            diagnostic.source_root.as_ref().is_some_and(|root| {
                target["src_path"]
                    .as_str()
                    .is_some_and(|path| root == std::path::Path::new(path))
            })
        })
        .or_else(|| {
            package.targets.iter().find(|t| {
                t["src_path"].as_str().is_some_and(|path| {
                    package
                        .sources
                        .iter()
                        .any(|s| s == std::path::Path::new(path))
                })
            })
        })
        .ok_or("package has no product target for editor diagnostics")?;
    let source = fs::read_to_string(&diagnostic.file).map_err(|e| e.to_string())?;
    let (byte_start, byte_end, text) = location(&source, diagnostic.line, diagnostic.column)?;
    Ok(json!({
        "reason":"compiler-message", "package_id":package.id,
        "manifest_path":package.directory.join("Cargo.toml"), "target":target,
        "message":{
            "$message_type":"diagnostic", "message":diagnostic.message,
            "code":{"code":diagnostic.rule,"explanation":null}, "level":"error",
            "spans":[{
                "file_name":diagnostic.file, "byte_start":byte_start, "byte_end":byte_end,
                "line_start":diagnostic.line, "line_end":diagnostic.line,
                "column_start":diagnostic.column, "column_end":diagnostic.column + usize::from(byte_end > byte_start),
                "is_primary":true, "text":[{"text":text,"highlight_start":diagnostic.column,
                    "highlight_end":diagnostic.column + usize::from(byte_end > byte_start)}],
                "label":"GDP syntax convention", "suggested_replacement":null,
                "suggestion_applicability":null, "expansion":null
            }], "children":[], "rendered":format!("error[{}]: {}\n  --> {}:{}:{}\n", diagnostic.rule, diagnostic.message, diagnostic.file.display(), diagnostic.line, diagnostic.column)
        }
    }))
}

fn location(source: &str, line: usize, column: usize) -> Result<(usize, usize, &str), String> {
    let mut offset = 0;
    for (index, text) in source.split_inclusive('\n').enumerate() {
        if index + 1 == line {
            let text = text.trim_end_matches(['\r', '\n']);
            if column == 0 || column > text.chars().count() + 1 {
                return Err("diagnostic column is outside the source line".into());
            }
            let start = text
                .chars()
                .take(column - 1)
                .map(char::len_utf8)
                .sum::<usize>();
            let width = text[start..].chars().next().map_or(0, char::len_utf8);
            return Ok((offset + start, offset + start + width, text));
        }
        offset += text.len();
    }
    Err("diagnostic line is outside the source file".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_format_is_normalized_before_calling_cargo() {
        let args = ["--workspace", "--message-format", "cargo-json", "--locked"].map(str::to_owned);
        let (format, forwarded) = parse(&args).unwrap();
        assert!(format == Format::CargoJson);
        assert_eq!(
            forwarded,
            ["--workspace", "--message-format=json", "--locked"]
        );
        assert!(parse(&["--message-format".into()]).is_err());
        assert!(
            parse(&["--message-format=json-render-diagnostics".into()])
                .unwrap()
                .0
                == Format::Json
        );
    }

    #[test]
    fn source_offsets_count_unicode_and_crlf_correctly() {
        assert_eq!(
            location("// π\r\nαβ fn x() {}\n", 2, 4).unwrap(),
            (12, 13, "αβ fn x() {}")
        );
        assert!(location("a", 1, 3).is_err());
        assert!(location("a", 0, 1).is_err());
    }
}
