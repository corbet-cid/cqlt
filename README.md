# cqlt

Deterministic organization and repository quality checks over saved evidence.
A reusable Rust library, consumed by [ccid](https://git.corbet.ch/corbet-cid/ccid).

cqlt also owns a subordinate [Vale prose backend](docs/prose.md), including its
writing rules, invocation protocol and stable evidence report. ccid provides
bounded execution. [Semantic quality research](studies/semantic-quality.md)
assesses TypeSafe AI Jev and existing tools for evidence-based judgments.

## Contract

`evaluate(&Snapshot, &Policy)` is pure: no network, clock, model, filesystem or
provider mutations. Sort organizations, repositories, topics and exceptions;
validate identities and completeness; evaluate the versioned rule catalogue;
apply exact, reasoned exceptions; sort checks by subject and rule. Reports bind
the normalized evidence and policy with SHA-256 and include the ruleset ID.
Identical inputs produce byte-identical serialized reports. Change the ruleset
ID when semantics change.

Every check records pass, fail, unknown, not-applicable or waived, together with
evidence, severity and remediation. Unknown evidence always returns exit 2;
quality failures return 1; success returns 0. Empty scopes, duplicate identities,
unsupported schemas, unknown rules and malformed policies are errors. Provider
counts must match collected records. Completeness covers the collector's visible
scope, never repositories hidden from its credentials.

## Presentation rules v1

| Rule | Default | Requirement |
|---|---|---|
| `org.description`, `repo.description` | error | Single-line nonempty purpose; no redundant whitespace, name-only text or exact TODO/TBD/WIP/placeholder/coming-soon marker |
| `org.name` | warning | Nonempty organization display name |
| `org.profile` | error | Nonempty organization profile README |
| `org.website`, `repo.homepage` | warning | If supplied, absolute HTTP(S) syntax without whitespace or credentials |
| `repo.readme` | error | Nonempty README at an inspected supported location |
| `repo.license` | error | Explicit nonempty root license in public repositories |
| `repo.topics` | warning | Topics on active public original projects, excluding `.github`/`.profile` |
| `repo.name-collision` | warning | Review equal names across active original repositories, excluding `.github`/`.profile` |

Description markers match the entire text, case-insensitively, with an optional
trailing period. These checks establish observable presentation hygiene, not
prose quality, legal suitability, link reachability, activity or code quality.
Equal names are review candidates, not proof of duplicate code. Archived and
forked repositories still receive basic description/document checks, but topics
and collision rules do not pressure them to look like active original projects.
Private/internal repositories do not require a public license. An explicit
exception records a reviewed intentional absence; unknowns cannot be waived.

The rule catalogue is generic. Keep organization charters, inventories, private
repositories and policy exceptions in the caller's private configuration.

```rust
# fn example(snapshot: cqlt::Snapshot) -> Result<(), String> {
let report = cqlt::evaluate(&snapshot, &cqlt::Policy::default())?;
assert_eq!(report.ruleset, cqlt::RULESET);
let status = report.exit_code(cqlt::Severity::Error);
# Ok(())
# }
```

Policy JSON is schema 1 with optional `severities` (rule → `error`/`warning`)
and `exceptions` (objects with exact `subject`, `rule`, nonempty `reason`).
Provider-specific collection belongs to adapters (GitHub, Forgejo, or other
forges); snapshots carry the source instance identity. Collection completeness cannot be downgraded or exempted. Reports retain
exceptions even when the underlying check passes, making obsolete waivers
visible for review. No baseline hides newly created repositories.

## Development

Use current stable Rust (minimum 1.89). Run `cargo fmt --all -- --check`,
`cargo test --locked`, and `cargo clippy --all-targets --locked -- -D warnings`.
The tests cover deterministic ordering, truncation, unknowns, exact exceptions,
policy typos, threshold behavior, private/fork/archive applicability and malformed
input. CI uses the same commands; no credentials or live organization data are
required.

## License

Copyright 2026 Julian Y. Richard Corbet. Licensed under
`LGPL-3.0-only WITH LGPL-3.0-linking-exception`; see [LICENSE.md](LICENSE.md).
