use cqlt::*;

fn fixture() -> Snapshot {
    let doc = || Document::Present {
        path: "README.md".into(),
        bytes: 80,
    };
    Snapshot {
        schema: 1,
        source: "https://forge.example/api".into(),
        collected_at: "2026-01-01T00:00:00Z".into(),
        scope: vec!["example".into()],
        complete: true,
        errors: vec![],
        organizations: vec![Organization {
            login: "example".into(),
            name: Some("Example tools".into()),
            description: Some("Tools for authors".into()),
            website: None,
            profile: doc(),
            repository_count: 1,
            repositories: vec![Repository {
                name: "editor".into(),
                full_name: "example/editor".into(),
                description: Some("A text editor for authors".into()),
                homepage: None,
                visibility: Visibility::Public,
                fork: false,
                archived: false,
                revision: Some("a".repeat(40)),
                topics: vec!["editing".into()],
                readme: doc(),
                license: Document::Present {
                    path: "LICENSE".into(),
                    bytes: 10,
                },
            }],
        }],
    }
}

#[test]
fn clean_evidence_passes_and_missing_fields_fail() {
    let mut snapshot = fixture();
    assert_eq!(
        evaluate(&snapshot, &Policy::default())
            .unwrap()
            .exit_code(Severity::Warning),
        0
    );
    snapshot.organizations[0].repositories[0].description = None;
    snapshot.organizations[0].profile = Document::Missing;
    let report = evaluate(&snapshot, &Policy::default()).unwrap();
    assert_eq!(report.exit_code(Severity::Error), 1);
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|c| c.status == Status::Fail)
            .count(),
        2
    );
}

#[test]
fn collection_failures_and_truncation_cannot_pass_or_be_waived() {
    for case in 0..5 {
        let mut snapshot = fixture();
        match case {
            0 => snapshot.complete = false,
            1 => snapshot
                .errors
                .push("Provider returned partial data".into()),
            2 => snapshot.organizations[0].repository_count = 2,
            3 => snapshot.scope.push("unavailable".into()),
            _ => {
                snapshot.organizations[0].profile = Document::Unknown {
                    reason: "Forbidden".into(),
                }
            }
        }
        let policy = Policy {
            exceptions: vec![Exception {
                subject: "example".into(),
                rule: "org.profile".into(),
                reason: "Deliberately omitted".into(),
            }],
            ..Policy::default()
        };
        assert_eq!(
            evaluate(&snapshot, &policy)
                .unwrap()
                .exit_code(Severity::Error),
            2
        );
    }
}

#[test]
fn ordering_does_not_change_report_or_identity() {
    let mut snapshot = fixture();
    let mut other = snapshot.organizations[0].clone();
    other.login = "another".into();
    other.repositories[0].full_name = "another/editor".into();
    snapshot.organizations[0].repositories[0]
        .topics
        .extend(["rust".into(), "editing".into()]);
    snapshot.scope.push("another".into());
    snapshot.organizations.push(other);
    let before = serde_json::to_vec(&evaluate(&snapshot, &Policy::default()).unwrap()).unwrap();
    snapshot.scope.reverse();
    snapshot.organizations.reverse();
    for org in &mut snapshot.organizations {
        org.repositories.reverse();
        for repo in &mut org.repositories {
            repo.topics.reverse();
        }
    }
    let after = serde_json::to_vec(&evaluate(&snapshot, &Policy::default()).unwrap()).unwrap();
    assert_eq!(before, after);
}

#[test]
fn exceptions_are_exact_reasoned_and_typo_checked() {
    let mut snapshot = fixture();
    snapshot.organizations[0].repositories[0].description = None;
    let mut policy = Policy {
        exceptions: vec![Exception {
            subject: "example/editor".into(),
            rule: "repo.description".into(),
            reason: "Migration tracked separately".into(),
        }],
        ..Policy::default()
    };
    let report = evaluate(&snapshot, &policy).unwrap();
    assert_eq!(report.exit_code(Severity::Error), 0);
    assert_eq!(
        report
            .checks
            .iter()
            .filter(|c| c.status == Status::Waived)
            .count(),
        1
    );
    policy.exceptions[0].reason.clear();
    assert!(evaluate(&snapshot, &policy).is_err());
    policy.exceptions.clear();
    policy
        .severities
        .insert("repo.descripton".into(), Severity::Warning);
    assert!(evaluate(&snapshot, &policy).is_err());
    policy.severities.clear();
    policy.exceptions.push(Exception {
        subject: "example/*".into(),
        rule: "repo.description".into(),
        reason: "Wildcard".into(),
    });
    assert!(evaluate(&snapshot, &policy).is_err());
}

#[test]
fn private_licenses_forks_and_archives_have_explicit_applicability() {
    for kind in 0..3 {
        let mut snapshot = fixture();
        let repo = &mut snapshot.organizations[0].repositories[0];
        repo.topics.clear();
        match kind {
            0 => {
                repo.visibility = Visibility::Private;
                repo.license = Document::Missing;
            }
            1 => repo.fork = true,
            _ => repo.archived = true,
        }
        assert_eq!(
            evaluate(&snapshot, &Policy::default())
                .unwrap()
                .exit_code(Severity::Warning),
            0
        );
    }
}

#[test]
fn thresholds_do_not_turn_unknowns_into_warnings() {
    let mut snapshot = fixture();
    snapshot.organizations[0].repositories[0].topics.clear();
    let report = evaluate(&snapshot, &Policy::default()).unwrap();
    assert_eq!(report.exit_code(Severity::Error), 0);
    assert_eq!(report.exit_code(Severity::Warning), 1);
    snapshot.organizations[0].repositories[0].license = Document::Unknown {
        reason: "timeout".into(),
    };
    assert_eq!(
        evaluate(&snapshot, &Policy::default())
            .unwrap()
            .exit_code(Severity::Error),
        2
    );
}

#[test]
fn invalid_inputs_do_not_create_a_green_empty_report() {
    let mut snapshot = fixture();
    snapshot.scope.clear();
    assert!(evaluate(&snapshot, &Policy::default()).is_err());
    let mut snapshot = fixture();
    let duplicate = snapshot.organizations[0].repositories[0].clone();
    snapshot.organizations[0].repositories.push(duplicate);
    assert!(evaluate(&snapshot, &Policy::default()).is_err());
    let mut snapshot = fixture();
    snapshot.organizations[0].repositories[0].full_name = "different/editor".into();
    assert!(evaluate(&snapshot, &Policy::default()).is_err());
    assert!(serde_json::from_str::<Policy>(r#"{"schema":1,"exeptions":[]}"#).is_err());
}

#[test]
fn description_rules_are_literal_and_unicode_safe() {
    for text in [
        "",
        "  ",
        "TBD",
        "Coming soon.",
        "editor",
        "Two  spaces",
        "Line\nbreak",
        "\u{1b}terminal",
    ] {
        let mut snapshot = fixture();
        snapshot.organizations[0].repositories[0].description = Some(text.into());
        assert_eq!(
            evaluate(&snapshot, &Policy::default())
                .unwrap()
                .exit_code(Severity::Error),
            1,
            "{text:?}"
        );
    }
    let mut snapshot = fixture();
    snapshot.organizations[0].description = Some("Outils pour écrire — 中文".into());
    assert_eq!(
        evaluate(&snapshot, &Policy::default())
            .unwrap()
            .exit_code(Severity::Error),
        0
    );
}

#[test]
fn empty_documents_and_bad_homepage_syntax_are_reported() {
    let mut snapshot = fixture();
    snapshot.organizations[0].repositories[0].readme = Document::Present {
        path: "README.md".into(),
        bytes: 0,
    };
    assert_eq!(
        evaluate(&snapshot, &Policy::default())
            .unwrap()
            .exit_code(Severity::Error),
        1
    );
    for url in [
        "example.org",
        "https://",
        "https://user:password@example.org",
        "https://example.org/a b",
    ] {
        let mut snapshot = fixture();
        snapshot.organizations[0].website = Some(url.into());
        assert_eq!(
            evaluate(&snapshot, &Policy::default())
                .unwrap()
                .exit_code(Severity::Warning),
            1
        );
    }
}
