//! cqlt owns the Vale backend: input selection, rules, invocation and report.
//! The caller materializes `Plan::files()` in a fresh private directory and
//! supplies bounded execution there. No rule downloads or project config loads.
mod model;
mod protocol;
pub use model::*;

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const RULESET: &str = "cqlt-prose-v2";
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_DOCUMENTS: usize = 4096;
const CONFIG: &str =
    "StylesPath = styles\nMinAlertLevel = suggestion\n\n[*.{md,txt}]\nBasedOnStyles = Cqlt\n";
const STYLES: &[(&str, &str)] = &[
    (
        "Cqlt.Repetition",
        include_str!("../../styles/Cqlt/Repetition.yml"),
    ),
    (
        "Cqlt.VagueClaims",
        include_str!("../../styles/Cqlt/VagueClaims.yml"),
    ),
    (
        "Cqlt.Wordiness",
        include_str!("../../styles/Cqlt/Wordiness.yml"),
    ),
];

pub(super) fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json_hash(value: &impl Serialize) -> String {
    hash(serde_json::to_vec(value).expect("serializable prose model"))
}

pub struct Plan {
    documents: BTreeMap<String, Text>,
    input_sha256: String,
}

impl Plan {
    pub fn new(mut documents: Vec<Text>) -> Result<Self, String> {
        if documents.is_empty() || documents.len() > MAX_DOCUMENTS {
            return Err(format!("Provide 1..={MAX_DOCUMENTS} prose documents"));
        }
        documents.sort_by(|a, b| a.subject.cmp(&b.subject));
        let mut subjects = BTreeSet::new();
        let mut bytes = 0usize;
        for doc in &documents {
            if doc.subject.trim().is_empty()
                || doc.subject.chars().any(char::is_control)
                || !subjects.insert(&doc.subject)
            {
                return Err("Prose subjects must be nonempty, unique and free of controls".into());
            }
            bytes = bytes.saturating_add(doc.text.len());
            if bytes > MAX_INPUT_BYTES || doc.text.contains('\0') {
                return Err("Prose exceeds 16 MiB or contains NUL bytes".into());
            }
        }
        let input_sha256 = json_hash(&documents);
        Ok(Self {
            documents: documents
                .into_iter()
                .enumerate()
                .map(|(i, d)| (format!("documents/{i:04}.{}", d.format.extension()), d))
                .collect(),
            input_sha256,
        })
    }

    /// Relative, generated paths only; source identifiers cannot escape a root.
    pub fn files(&self) -> BTreeMap<String, String> {
        let mut files: BTreeMap<_, _> = self
            .documents
            .iter()
            .map(|(p, d)| (p.clone(), d.text.clone()))
            .collect();
        files.insert(".vale.ini".into(), CONFIG.into());
        for (name, style) in STYLES {
            files.insert(
                format!("styles/{}.yml", name.replace('.', "/")),
                (*style).into(),
            );
        }
        files
    }

    /// The executor must run in the materialized workspace, return stdout only
    /// on successful execution, and propagate missing tools/timeouts/failures.
    /// It must bound time and captured output. cqlt never installs executables.
    pub fn run(
        &self,
        mut execute: impl FnMut(&[String]) -> Result<String, String>,
    ) -> Result<Report, String> {
        let version = execute(&["vale".into(), "--version".into()])?;
        let argv = [
            "vale",
            "--no-global",
            "--no-exit",
            "--output=JSON",
            "--counts",
            "--normalize",
            "--relative",
            "--config=.vale.ini",
            "documents",
        ]
        .map(str::to_owned);
        let output = execute(&argv)?;
        self.report(&version, &output)
    }

    /// Pure replay of the exact backend response against this input and policy.
    pub fn report(&self, engine_version: &str, output: &str) -> Result<Report, String> {
        protocol::report(self, engine_version, output)
    }
}

/// Select all descriptions from complete, validated forge evidence. Missing
/// descriptions become explicit empty-text findings, never silent exclusions.
pub fn descriptions(snapshot: &crate::Snapshot) -> Result<Vec<Text>, String> {
    let report = crate::evaluate(snapshot, &crate::Policy::default())?;
    if report
        .checks
        .iter()
        .any(|c| c.rule == "collection.complete" && c.status == crate::Status::Unknown)
    {
        return Err("Description audit requires complete collection evidence".into());
    }
    let mut documents = Vec::new();
    for org in &snapshot.organizations {
        documents.push(Text {
            subject: format!("{}#description", org.login),
            format: Format::Text,
            text: org.description.clone().unwrap_or_default(),
        });
        for repo in &org.repositories {
            documents.push(Text {
                subject: format!("{}#description", repo.full_name),
                format: Format::Text,
                text: repo.description.clone().unwrap_or_default(),
            });
        }
    }
    Ok(documents)
}
