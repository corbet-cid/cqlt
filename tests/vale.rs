//! Real backend contract: run explicitly with Vale installed on the CI worker.
use cqlt::prose::{Format, Level, Plan, Text};
use std::{fs, path::PathBuf, process::Command};

struct Workspace(PathBuf);
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "requires Vale >=3.23; CI runs this explicitly"]
fn actual_vale_checks_prose_preserves_markdown_and_replays_stably() {
    let path = std::env::temp_dir().join(format!("cqlt-vale-test-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let root = Workspace(path);
    let plan = Plan::new(vec![
        Text { subject:"readme".into(), format:Format::Markdown,
            text:"# Sample\n\nA revolutionary tool in order to check the the text.\n\n```text\nseamless seamless utilize\n```\n\n`world-class`\n".into() },
        Text { subject:"description".into(), format:Format::Text, text:"A world-class tool.".into() },
        Text { subject:"clean".into(), format:Format::Text, text:"A tool for checking repository descriptions.".into() },
        Text { subject:"code-boundary".into(), format:Format::Markdown,
            text:"Tools (`prepare` and `run`) and their reports.\n\nGo, go.\n".into() },
        Text { subject:"empty".into(), format:Format::Text, text:String::new() },
    ]).unwrap();
    for (path, content) in plan.files() {
        let path = root.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    let run = || {
        plan.run(|args| {
            let result = Command::new(&args[0])
                .args(&args[1..])
                .current_dir(&root.0)
                .output()
                .map_err(|e| e.to_string())?;
            if !result.status.success() {
                return Err(String::from_utf8_lossy(&result.stderr).into_owned());
            }
            String::from_utf8(result.stdout).map_err(|e| e.to_string())
        })
        .unwrap()
    };
    let first = run();
    assert_eq!(first.exit_code(Level::Warning), 1);
    assert_eq!(first.findings.len(), 5, "{:?}", first.findings);
    assert!(first.findings.iter().all(|f| f.subject != "clean"));
    assert!(first
        .findings
        .iter()
        .filter(|f| f.subject == "readme")
        .all(|f| f.line == 3));
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&run()).unwrap()
    );
}
