use cqlt::prose::{Format, Level, Plan, Text};
use serde_json::{json, Value};

fn text(subject: &str, text: &str) -> Text {
    Text {
        subject: subject.into(),
        format: Format::Text,
        text: text.into(),
    }
}

fn output() -> Value {
    json!({"files": {}, "counts": {
        "Cqlt.Repetition": 0, "Cqlt.VagueClaims": 0, "Cqlt.Wordiness": 0
    }})
}

#[test]
fn reports_are_stable_across_input_order_and_workspace_names() {
    let a = Plan::new(vec![text("z", "Tools for writers."), text("a", "")]).unwrap();
    let b = Plan::new(vec![text("a", ""), text("z", "Tools for writers.")]).unwrap();
    assert_eq!(a.files(), b.files());
    let report = a
        .report("vale version 3.23.0", &output().to_string())
        .unwrap();
    assert_eq!(report.exit_code(Level::Error), 1);
    assert_eq!(report.findings[0].rule, "cqlt.text.nonempty");
    assert_eq!(
        serde_json::to_vec(&report).unwrap(),
        serde_json::to_vec(
            &b.report("vale version 3.23.0", &output().to_string())
                .unwrap()
        )
        .unwrap()
    );
    let c = Plan::new(vec![text("z", "Tools for editors."), text("a", "")]).unwrap();
    assert_ne!(
        report.input_sha256,
        c.report("vale version 3.23.0", &output().to_string())
            .unwrap()
            .input_sha256
    );
}

#[test]
fn backend_failure_and_disabled_rules_never_pass() {
    let plan = Plan::new(vec![text("description", "Tools for writers.")]).unwrap();
    assert!(plan.run(|_| Err("missing vale".into())).is_err());
    for bad in ["{}", "[]", "not JSON", "{\"files\":{},\"counts\":{}}"] {
        assert!(plan.report("vale version 3.23.0", bad).is_err());
    }
    assert!(plan.report("", &output().to_string()).is_err());
    let mut value = output();
    value["counts"]["Cqlt.Repetition"] = json!(1);
    assert!(plan
        .report("vale version 3.23.0", &value.to_string())
        .is_err());
}

#[test]
fn malformed_findings_and_unrequested_documents_are_errors() {
    let plan = Plan::new(vec![text("description", "A seamless tool.")]).unwrap();
    let mut value = output();
    value["counts"]["Cqlt.VagueClaims"] = json!(1);
    value["files"]["documents/0000.txt"] = json!([{
        "Check":"Cqlt.VagueClaims", "Severity":"warning", "Message":"Give evidence.",
        "Match":"seamless", "Line":1, "Span":[3,10]
    }]);
    let report = plan
        .report("vale version 3.23.0", &value.to_string())
        .unwrap();
    assert_eq!(report.exit_code(Level::Warning), 1);
    assert_eq!(report.exit_code(Level::Error), 0);
    assert_eq!(report.findings[0].subject, "description");
    for (field, invalid) in [
        ("Severity", json!("fatal")),
        ("Line", json!(0)),
        ("Span", json!([2, 1])),
        ("Check", json!("Unknown.Rule")),
    ] {
        let mut bad = value.clone();
        bad["files"]["documents/0000.txt"][0][field] = invalid;
        assert!(plan
            .report("vale version 3.23.0", &bad.to_string())
            .is_err());
    }
    value["files"]["outside.txt"] = value["files"]["documents/0000.txt"].take();
    value["files"]
        .as_object_mut()
        .unwrap()
        .remove("documents/0000.txt");
    assert!(plan
        .report("vale version 3.23.0", &value.to_string())
        .is_err());
}

#[test]
fn identifiers_never_become_paths_and_input_scope_is_explicit() {
    let plan = Plan::new(vec![text("../../outside", "Tools for writers.")]).unwrap();
    assert!(plan.files().contains_key("documents/0000.txt"));
    assert!(plan.files().keys().all(|p| !p.contains("..")));
    assert!(Plan::new(vec![]).is_err());
    assert!(Plan::new(vec![text("a", "x"), text("a", "y")]).is_err());
    assert!(Plan::new(vec![text("bad\nsubject", "x")]).is_err());
    assert!(Plan::new(vec![text("a", "\0")]).is_err());
}

#[test]
fn cqlt_owns_the_exact_vale_protocol() {
    let plan = Plan::new(vec![text("description", "Tools for writers.")]).unwrap();
    let mut calls = Vec::new();
    let report = plan
        .run(|args| {
            calls.push(args.to_vec());
            Ok(if args.contains(&"--version".into()) {
                "vale version 3.23.0".into()
            } else {
                output().to_string()
            })
        })
        .unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls[1].contains(&"--no-global".into()));
    assert!(calls[1].contains(&"--counts".into()));
    assert!(!calls[1].contains(&"sync".into()));
    assert_eq!(report.exit_code(Level::Warning), 0);
}

#[test]
fn incomplete_forge_evidence_is_rejected_before_running_vale() {
    let snapshot = serde_json::from_value(json!({
        "schema":1,"source":"github:https://api.github.com","collected_at":"2026-01-01",
        "scope":["example"],"complete":false,"errors":["collection failed"],"organizations":[]
    }))
    .unwrap();
    assert!(cqlt::prose::descriptions(&snapshot).is_err());
}
