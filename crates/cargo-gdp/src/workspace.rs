use serde_json::Value;
use std::{collections::BTreeSet, env, path::PathBuf, process::Command};

pub struct Package {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
    pub sources: Vec<PathBuf>,
    pub targets: Vec<Value>,
}

pub fn selected(args: &[String]) -> Result<Vec<Package>, String> {
    let mut metadata_args = Vec::new();
    let mut requested = Vec::new();
    let mut excluded = Vec::new();
    let mut all = false;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "--workspace" => all = true,
            "-p" | "--package" | "--exclude" | "--manifest-path" | "--config" | "--features"
            | "-F" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                match arg.as_str() {
                    "--manifest-path" | "--config" | "--features" | "-F" => {
                        metadata_args.extend([arg.clone(), value.clone()])
                    }
                    "--exclude" => excluded.push(value.clone()),
                    _ => requested.push(value.clone()),
                }
            }
            "--offline" | "--locked" | "--frozen" | "--all-features" | "--no-default-features" => {
                metadata_args.push(arg.clone())
            }
            _ if arg.starts_with("--manifest-path=") => metadata_args.push(arg.clone()),
            _ if arg.starts_with("--config=") || arg.starts_with("--features=") => {
                metadata_args.push(arg.clone())
            }
            _ if arg.starts_with("--package=") => requested.push(arg[10..].to_owned()),
            _ if arg.starts_with("--exclude=") => excluded.push(arg[10..].to_owned()),
            _ if arg.starts_with("-p") && arg.len() > 2 => requested.push(arg[2..].to_owned()),
            _ => {} // cargo check validates and handles other forwarded options.
        }
        i += 1;
    }
    let output = Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .args(metadata_args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    select(&metadata, &requested, &excluded, all)
}

fn select(
    metadata: &Value,
    requested: &[String],
    excluded: &[String],
    all: bool,
) -> Result<Vec<Package>, String> {
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .ok_or("missing workspace_members")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let defaults: BTreeSet<&str> = metadata["workspace_default_members"]
        .as_array()
        .ok_or("missing workspace_default_members")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let mut found = BTreeSet::new();
    let mut result = Vec::new();
    for package in metadata["packages"].as_array().ok_or("missing packages")? {
        let id = package["id"].as_str().ok_or("missing package id")?;
        if !members.contains(id) {
            continue;
        }
        let name = package["name"].as_str().ok_or("missing package name")?;
        let matching: Vec<_> = requested
            .iter()
            .filter(|s| *s == name || *s == id)
            .collect();
        for spec in &matching {
            found.insert(spec.as_str());
        }
        let selected = all || !matching.is_empty() || requested.is_empty() && defaults.contains(id);
        if selected && !excluded.iter().any(|s| s == name || s == id) {
            let manifest = PathBuf::from(
                package["manifest_path"]
                    .as_str()
                    .ok_or("missing manifest path")?,
            );
            result.push(Package {
                id: id.to_owned(),
                name: name.to_owned(),
                directory: manifest
                    .parent()
                    .ok_or("manifest has no directory")?
                    .to_owned(),
                sources: package["targets"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter(|target| {
                        target["kind"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter_map(Value::as_str)
                            .any(|kind| {
                                matches!(
                                    kind,
                                    "lib"
                                        | "bin"
                                        | "proc-macro"
                                        | "example"
                                        | "rlib"
                                        | "cdylib"
                                        | "dylib"
                                        | "staticlib"
                                )
                            })
                    })
                    .filter_map(|target| target["src_path"].as_str().map(PathBuf::from))
                    .collect(),
                targets: package["targets"].as_array().cloned().unwrap_or_default(),
            });
        }
    }
    for spec in requested {
        if !found.contains(spec.as_str()) {
            return Err(format!(
                "syntax analysis supports exact local package names or IDs; unsupported selector {spec:?}"
            ));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selection_respects_defaults_packages_and_exclusions() {
        let m = serde_json::json!({"workspace_members":["a","b"], "workspace_default_members":["a"],
            "packages":[{"id":"a","name":"core","manifest_path":"/core/Cargo.toml"}, {"id":"b","name":"tool","manifest_path":"/tool/Cargo.toml"}]});
        assert_eq!(select(&m, &[], &[], false).unwrap()[0].name, "core");
        assert_eq!(
            select(&m, &["tool".into()], &[], false).unwrap()[0].name,
            "tool"
        );
        assert_eq!(
            select(&m, &[], &["core".into()], true).unwrap()[0].name,
            "tool"
        );
        assert!(select(&m, &["missing".into()], &[], false).is_err());
    }
}
