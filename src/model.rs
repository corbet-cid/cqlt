//! Forge-neutral evidence, policy and presentation reports.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: u32,
    /// Provider and instance identity supplied by the collector.
    pub source: String,
    pub collected_at: String,
    /// All organizations requested, including ones whose collection failed.
    pub scope: Vec<String>,
    pub complete: bool,
    pub errors: Vec<String>,
    pub organizations: Vec<Organization>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Organization {
    pub login: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub profile: Document,
    /// Count reported by the provider, compared against the collected records.
    pub repository_count: usize,
    pub repositories: Vec<Repository>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub visibility: Visibility,
    pub fork: bool,
    pub archived: bool,
    pub revision: Option<String>,
    pub topics: Vec<String>,
    pub readme: Document,
    pub license: Document,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    Private,
    Internal,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum Document {
    Present { path: String, bytes: u64 },
    Missing,
    Unknown { reason: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
    Waived,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema: u32,
    /// Overrides apply only to known rule IDs; unknown IDs are errors.
    #[serde(default)]
    pub severities: BTreeMap<String, Severity>,
    /// Exact subject/rule pairs, never globs. Unknown evidence cannot be waived.
    #[serde(default)]
    pub exceptions: Vec<Exception>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            schema: 1,
            severities: BTreeMap::new(),
            exceptions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Exception {
    pub subject: String,
    pub rule: String,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub subject: String,
    pub rule: String,
    pub severity: Severity,
    pub status: Status,
    pub evidence: String,
    pub remedy: String,
    pub exception: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema: u32,
    pub ruleset: String,
    pub snapshot_sha256: String,
    pub policy_sha256: String,
    pub organizations: usize,
    pub repositories: usize,
    pub checks: Vec<Check>,
}

impl Report {
    /// 0: meets the threshold, 1: quality failure, 2: incomplete evidence.
    pub fn exit_code(&self, fail_on: Severity) -> u8 {
        if self.checks.iter().any(|c| c.status == Status::Unknown) {
            return 2;
        }
        u8::from(self.checks.iter().any(|c| {
            c.status == Status::Fail
                && (fail_on == Severity::Warning || c.severity == Severity::Error)
        }))
    }
}
