use std::path::PathBuf;

/// A syntax finding with a stable rule identifier and one-based location.
pub struct Diagnostic {
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub rule: &'static str,
    pub message: &'static str,
    pub source_root: Option<PathBuf>,
}

impl Diagnostic {
    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "reason": "gdp-diagnostic", "schema_version": 1, "analysis": "syntax", "level": "error",
            "file": self.file, "line": self.line, "column": self.column,
            "code": self.rule, "message": self.message,
        })
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.file.display(),
            self.line,
            self.column,
            self.rule,
            self.message
        )
    }
}
