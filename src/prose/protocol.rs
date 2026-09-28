use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Output {
    files: BTreeMap<String, Vec<Alert>>,
    counts: BTreeMap<String, usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Alert {
    check: String,
    severity: Level,
    message: String,
    #[serde(rename = "Match")]
    matched: String,
    line: usize,
    span: [usize; 2],
}

pub(super) fn report(plan: &Plan, version: &str, output: &str) -> Result<Report, String> {
    let version = version.trim();
    if !version.starts_with("vale version ")
        || version.len() > 128
        || version.chars().any(char::is_control)
        || version == "vale version"
    {
        return Err("Missing or invalid Vale version identity".into());
    }
    if output.len() > MAX_INPUT_BYTES {
        return Err("Vale output exceeds 16 MiB".into());
    }
    let output: Output =
        serde_json::from_str(output).map_err(|e| format!("Invalid Vale JSON: {e}"))?;
    let mut observed: BTreeMap<String, usize> =
        STYLES.iter().map(|(n, _)| ((*n).into(), 0)).collect();
    if output.counts.keys().ne(observed.keys()) {
        return Err("Vale did not load exactly the cqlt prose rules".into());
    }
    let mut findings = Vec::new();
    for (path, alerts) in output.files {
        let doc = plan
            .documents
            .get(&path)
            .ok_or_else(|| format!("Unexpected Vale document: {path:?}"))?;
        for alert in alerts {
            let count = observed
                .get_mut(&alert.check)
                .ok_or("Unexpected Vale rule")?;
            *count += 1;
            if alert.severity != Level::Warning
                || alert.line == 0
                || alert.line > doc.text.lines().count()
                || alert.span[0] == 0
                || alert.span[1] < alert.span[0]
                || alert.matched.is_empty()
                || alert.message.trim().is_empty()
            {
                return Err("Invalid Vale finding location, severity or evidence".into());
            }
            findings.push(Finding {
                subject: doc.subject.clone(),
                line: alert.line,
                span: alert.span,
                rule: alert.check,
                severity: alert.severity,
                matched: alert.matched,
                message: alert.message,
            });
        }
    }
    if observed != output.counts {
        return Err("Vale alert counts disagree with its findings".into());
    }
    for doc in plan.documents.values().filter(|d| d.text.trim().is_empty()) {
        findings.push(Finding {
            subject: doc.subject.clone(),
            line: 1,
            span: [1, 1],
            rule: "cqlt.text.nonempty".into(),
            severity: Level::Error,
            matched: String::new(),
            message: "Write a concrete description of the purpose.".into(),
        });
    }
    findings.sort();
    Ok(Report {
        schema: 1,
        ruleset: RULESET.into(),
        engine: version.into(),
        input_sha256: plan.input_sha256.clone(),
        policy_sha256: json_hash(&(CONFIG, STYLES)),
        documents: plan
            .documents
            .values()
            .map(|d| CheckedText {
                subject: d.subject.clone(),
                format: d.format,
                content_sha256: hash(&d.text),
            })
            .collect(),
        findings,
    })
}
