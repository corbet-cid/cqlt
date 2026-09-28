//! Stable, provider-independent prose inputs and reports.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Markdown,
    Text,
}

impl Format {
    pub(super) fn extension(self) -> &'static str {
        match self {
            Self::Markdown => "md",
            Self::Text => "txt",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    /// Stable identifier, never interpreted as a filesystem path.
    pub subject: String,
    pub format: Format,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Suggestion,
    Warning,
    Error,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub subject: String,
    pub line: usize,
    pub span: [usize; 2],
    pub rule: String,
    pub severity: Level,
    pub matched: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CheckedText {
    pub subject: String,
    pub format: Format,
    pub content_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    pub schema: u32,
    pub ruleset: String,
    pub engine: String,
    pub input_sha256: String,
    pub policy_sha256: String,
    pub documents: Vec<CheckedText>,
    pub findings: Vec<Finding>,
}

impl Report {
    /// Execution/protocol errors are returned before a report exists (CLI exit 2).
    pub fn exit_code(&self, threshold: Level) -> u8 {
        u8::from(self.findings.iter().any(|f| f.severity >= threshold))
    }
}
