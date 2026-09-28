//! Pure presentation checks over collected forge metadata.
use crate::model::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const RULESET: &str = "cqlt-presentation-v1";

pub const RULES: &[(&str, Severity)] = &[
    ("org.description", Severity::Error),
    ("org.name", Severity::Warning),
    ("org.website", Severity::Warning),
    ("org.profile", Severity::Error),
    ("repo.description", Severity::Error),
    ("repo.homepage", Severity::Warning),
    ("repo.readme", Severity::Error),
    ("repo.license", Severity::Error),
    ("repo.topics", Severity::Warning),
    ("repo.name-collision", Severity::Warning),
];

fn digest(value: &impl Serialize) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("serializable model"))
    )
}

fn check(subject: &str, rule: &str, status: Status, evidence: String, remedy: &str) -> Check {
    Check {
        subject: subject.into(),
        rule: rule.into(),
        severity: RULES
            .iter()
            .find(|(id, _)| *id == rule)
            .map_or(Severity::Error, |(_, s)| *s),
        status,
        evidence,
        remedy: remedy.into(),
        exception: None,
    }
}

fn description(subject: &str, rule: &str, value: &Option<String>, name: &str) -> Check {
    let text = value.as_deref().unwrap_or("");
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let placeholder = ["todo", "tbd", "wip", "placeholder", "coming soon"]
        .contains(&normalized.to_ascii_lowercase().trim_end_matches('.'));
    let valid = !normalized.is_empty()
        && !placeholder
        && !normalized.eq_ignore_ascii_case(name)
        && text == normalized
        && !text.chars().any(char::is_control);
    check(
        subject,
        rule,
        if valid { Status::Pass } else { Status::Fail },
        format!("description={text:?}"),
        "Write a single-line purpose statement; remove placeholders and redundant whitespace.",
    )
}

fn website(subject: &str, rule: &str, value: &Option<String>) -> Check {
    let text = value.as_deref().unwrap_or("");
    let host = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))
        .map(|s| s.split(['/', '?', '#']).next().unwrap_or(""));
    // This is a presentation syntax rule, not a URL parser or reachability probe.
    let valid = host.is_some_and(|h| !h.is_empty() && !h.contains('@'))
        && !text.chars().any(|c| c.is_whitespace() || c.is_control())
        && !text.contains('\\');
    check(
        subject,
        rule,
        if text.is_empty() {
            Status::NotApplicable
        } else if valid {
            Status::Pass
        } else {
            Status::Fail
        },
        format!("url={text:?}"),
        "Use an absolute http:// or https:// link without credentials or whitespace.",
    )
}

fn document(subject: &str, rule: &str, doc: &Document, remedy: &str) -> Check {
    let (status, evidence) = match doc {
        Document::Present { path, bytes } => (
            if *bytes > 0 && !path.trim().is_empty() {
                Status::Pass
            } else {
                Status::Fail
            },
            format!("{path}: {bytes} bytes"),
        ),
        Document::Missing => (
            Status::Fail,
            "Document absent from inspected revision".into(),
        ),
        Document::Unknown { reason } => (Status::Unknown, reason.clone()),
    };
    check(subject, rule, status, evidence, remedy)
}

/// Evaluate a snapshot under an explicit, versioned policy. Collection order has
/// no effect on results or digests. Invalid identities/policies are rejected.
pub fn evaluate(snapshot: &Snapshot, policy: &Policy) -> Result<Report, String> {
    if snapshot.schema != 1 || policy.schema != 1 {
        return Err("Unsupported schema".into());
    }
    if snapshot.scope.is_empty() {
        return Err("Empty audit scope".into());
    }
    let mut snapshot = snapshot.clone();
    snapshot.scope.sort();
    snapshot.errors.sort();
    snapshot.organizations.sort_by(|a, b| a.login.cmp(&b.login));
    let mut scope = BTreeSet::new();
    for login in &snapshot.scope {
        if !valid_login(login) || !scope.insert(login.to_ascii_lowercase()) {
            return Err(format!("Invalid or duplicate scope identity: {login}"));
        }
    }
    let mut subjects = BTreeSet::new();
    let mut found_orgs = BTreeSet::new();
    let mut inventory = snapshot.complete && snapshot.errors.is_empty();
    let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for org in &mut snapshot.organizations {
        let login = org.login.to_ascii_lowercase();
        if !scope.contains(&login) || !found_orgs.insert(login) {
            return Err(format!(
                "Unexpected or duplicate organization: {}",
                org.login
            ));
        }
        subjects.insert(org.login.clone());
        inventory &= org.repository_count == org.repositories.len();
        org.repositories
            .sort_by(|a, b| a.full_name.cmp(&b.full_name));
        let mut repo_ids = BTreeSet::new();
        for repo in &mut org.repositories {
            if repo.name.is_empty()
                || repo.name.contains('/')
                || repo.full_name != format!("{}/{}", org.login, repo.name)
                || !repo_ids.insert(repo.full_name.to_ascii_lowercase())
            {
                return Err(format!(
                    "Invalid or duplicate repository: {}",
                    repo.full_name
                ));
            }
            if let Some(revision) = &repo.revision {
                if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(format!("Invalid revision for {}", repo.full_name));
                }
            }
            subjects.insert(repo.full_name.clone());
            repo.topics.sort();
            repo.topics.dedup();
            if !matches!(repo.name.as_str(), ".github" | ".profile") && !repo.fork && !repo.archived
            {
                names
                    .entry(repo.name.to_ascii_lowercase())
                    .or_default()
                    .push(repo.full_name.clone());
            }
        }
    }
    inventory &= found_orgs == scope;
    let mut policy = policy.clone();
    policy
        .exceptions
        .sort_by(|a, b| (&a.subject, &a.rule).cmp(&(&b.subject, &b.rule)));
    let known = |id: &str| RULES.iter().any(|(rule, _)| *rule == id);
    if policy.severities.keys().any(|rule| !known(rule)) {
        return Err("Unknown policy rule".into());
    }
    let mut exceptions = BTreeMap::new();
    for exception in &policy.exceptions {
        if !known(&exception.rule)
            || !subjects.contains(&exception.subject)
            || exception.reason.trim().is_empty()
            || exceptions
                .insert(
                    (exception.subject.as_str(), exception.rule.as_str()),
                    &exception.reason,
                )
                .is_some()
        {
            return Err(format!(
                "Invalid, duplicate or out-of-scope exception: {}/{}",
                exception.subject, exception.rule
            ));
        }
    }
    let mut checks = vec![check(
        "inventory",
        "collection.complete",
        if inventory {
            Status::Pass
        } else {
            Status::Unknown
        },
        format!(
            "requested={}; collected={}; complete={}; errors={:?}",
            snapshot.scope.len(),
            snapshot.organizations.len(),
            snapshot.complete,
            snapshot.errors
        ),
        "Recollect all requested organizations and repository pages with sufficient read access.",
    )];
    for org in &snapshot.organizations {
        let id = &org.login;
        checks.push(description(id, "org.description", &org.description, id));
        checks.push(check(
            id,
            "org.name",
            if org.name.as_deref().is_some_and(|s| !s.trim().is_empty()) {
                Status::Pass
            } else {
                Status::Fail
            },
            format!("name={:?}", org.name),
            "Set a readable organization display name.",
        ));
        checks.push(website(id, "org.website", &org.website));
        checks.push(document(
            id,
            "org.profile",
            &org.profile,
            "Add a nonempty organization profile README using the forge's supported location.",
        ));
        for repo in &org.repositories {
            let id = &repo.full_name;
            checks.push(description(
                id,
                "repo.description",
                &repo.description,
                &repo.name,
            ));
            checks.push(website(id, "repo.homepage", &repo.homepage));
            checks.push(document(
                id,
                "repo.readme",
                &repo.readme,
                "Add a nonempty README describing purpose and entry points.",
            ));
            let mut license = document(id,"repo.license",&repo.license,"Add the project's chosen license at repository root; never infer a license from visibility.");
            if repo.visibility != Visibility::Public {
                license.status = Status::NotApplicable;
                license.evidence =
                    "Private/internal repository: no public-license requirement".into();
            }
            checks.push(license);
            let discoverable = repo.visibility == Visibility::Public
                && !repo.archived
                && !repo.fork
                && !matches!(repo.name.as_str(), ".github" | ".profile");
            checks.push(check(
                id,
                "repo.topics",
                if !discoverable {
                    Status::NotApplicable
                } else if repo.topics.is_empty() {
                    Status::Fail
                } else {
                    Status::Pass
                },
                format!("topics={:?}", repo.topics),
                "Add relevant topics to make an active public project discoverable.",
            ));
            let collisions = names.get(&repo.name.to_ascii_lowercase()).filter(|_| {
                !repo.archived
                    && !repo.fork
                    && !matches!(repo.name.as_str(), ".github" | ".profile")
            });
            checks.push(check(id,"repo.name-collision",if collisions.is_some_and(|n| n.len()>1) { Status::Fail } else { Status::Pass },
                format!("active original repositories with this name={:?}",collisions.cloned().unwrap_or_default()),
                "Review ownership and canonical home; equal names alone do not prove duplicate code."));
        }
    }
    for exception in &policy.exceptions {
        if !checks
            .iter()
            .any(|c| c.subject == exception.subject && c.rule == exception.rule)
        {
            return Err(format!(
                "Exception does not identify an applicable check: {}/{}",
                exception.subject, exception.rule
            ));
        }
    }
    for c in &mut checks {
        if let Some(severity) = policy.severities.get(&c.rule) {
            c.severity = *severity;
        }
        if let Some(reason) = exceptions.get(&(c.subject.as_str(), c.rule.as_str())) {
            c.exception = Some((*reason).clone());
            if c.status == Status::Fail {
                c.status = Status::Waived;
            }
        }
    }
    checks.sort_by(|a, b| (&a.subject, &a.rule).cmp(&(&b.subject, &b.rule)));
    Ok(Report {
        schema: 1,
        ruleset: RULESET.into(),
        snapshot_sha256: digest(&snapshot),
        policy_sha256: digest(&policy),
        organizations: snapshot.organizations.len(),
        repositories: snapshot
            .organizations
            .iter()
            .map(|o| o.repositories.len())
            .sum(),
        checks,
    })
}

pub fn valid_login(login: &str) -> bool {
    !login.is_empty()
        && login.len() <= 255
        && login != "."
        && login != ".."
        && login
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
