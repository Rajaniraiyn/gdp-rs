use crate::rules::inspect;
use crate::{diagnostic::Diagnostic, workspace::Package};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use syn::Item;

pub fn scan_package(package: &Package, diagnostics: &mut Vec<Diagnostic>) -> Result<(), String> {
    let root = &package.directory;
    let mut visited = BTreeSet::new();
    // Traverse declared crate roots first so Rust's crate-root module paths are
    // established before scanning any leftover source files.
    for source in &package.sources {
        if source.exists() {
            scan_file(
                source,
                source.parent().ok_or("source has no parent")?,
                &mut visited,
                diagnostics,
            )?;
        }
    }
    // Test fixtures intentionally contain rejected declarations; check product
    // sources and examples rather than treating those fixtures as product code.
    for directory in ["src", "examples"] {
        let path = root.join(directory);
        if path.exists() {
            scan_directory(&path, &mut visited, diagnostics)?;
        }
    }
    Ok(())
}

fn scan_directory(
    path: &Path,
    visited: &mut BTreeSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let ty = entry.file_type().map_err(|e| e.to_string())?;
        if ty.is_symlink() {
            continue;
        }
        let path = entry.path();
        if ty.is_dir() {
            scan_directory(&path, visited, diagnostics)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let parent = path.parent().ok_or("source has no parent")?;
            let directory = if path
                .file_name()
                .is_some_and(|name| matches!(name.to_str(), Some("lib.rs" | "main.rs" | "mod.rs")))
            {
                parent.to_owned()
            } else {
                parent.join(path.file_stem().ok_or("source has no name")?)
            };
            scan_file(&path, &directory, visited, diagnostics)?;
        }
    }
    Ok(())
}

fn scan_file(
    path: &Path,
    module_dir: &Path,
    visited: &mut BTreeSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    // Do not follow a symlink into sources outside the chosen source boundary.
    if fs::symlink_metadata(path)
        .map_err(|e| e.to_string())?
        .file_type()
        .is_symlink()
    {
        return Ok(());
    }
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("{}: {e}", path.display()))?;
    if !visited.insert(canonical) {
        return Ok(());
    }
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    for (line, column, rule, message) in inspect(&file.items) {
        diagnostics.push(Diagnostic {
            file: path.to_owned(),
            line,
            column,
            rule,
            message,
        });
    }
    scan_modules(
        &file.items,
        path.parent().ok_or("source has no parent")?,
        module_dir,
        false,
        visited,
        diagnostics,
    )
}

fn scan_modules(
    items: &[Item],
    file_directory: &Path,
    directory: &Path,
    inline: bool,
    visited: &mut BTreeSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(), String> {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        let name = module.ident.to_string().trim_start_matches("r#").to_owned();
        let explicit = module.attrs.iter().find_map(|attribute| {
            if !attribute.path().is_ident("path") {
                return None;
            }
            let syn::Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            let syn::Expr::Lit(literal) = &value.value else {
                return None;
            };
            let syn::Lit::Str(path) = &literal.lit else {
                return None;
            };
            // In an outlined module, #[path] is relative to the source file's
            // parent. Inside an inline module it uses the logical directory.
            Some(if inline { directory } else { file_directory }.join(path.value()))
        });
        if let Some((_, children)) = &module.content {
            let nested = explicit.unwrap_or_else(|| directory.join(&name));
            scan_modules(
                children,
                file_directory,
                &nested,
                true,
                visited,
                diagnostics,
            )?;
            continue;
        }
        let file = explicit.unwrap_or_else(|| {
            let file = directory.join(format!("{name}.rs"));
            if file.exists() {
                file
            } else {
                directory.join(&name).join("mod.rs")
            }
        });
        // A module behind a disabled cfg may legitimately have no source file.
        if !file.exists() {
            continue;
        }
        let parent = file.parent().ok_or("module has no parent")?;
        let nested = if file.file_name().is_some_and(|name| name == "mod.rs") {
            parent.to_owned()
        } else {
            parent.join(file.file_stem().ok_or("module has no name")?)
        };
        scan_file(&file, &nested, visited, diagnostics)?;
    }
    Ok(())
}
