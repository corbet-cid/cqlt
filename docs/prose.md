# Prose quality

cqlt owns the prose policy and its subordinate Vale backend. Consumers such as
ccid provide files and bounded process execution; they do not interpret Vale
output or duplicate its rules. Metadata evaluation remains independent and pure.

## Writing standard

An organization description explains what belongs there. A repository description
states what the software does and its main use. A README opening explains the
audience, purpose and maturity before implementation details. Use specific,
evidence-supported language. Different audiences can need different wording;
their descriptions must agree on names, capabilities, ownership and status.

The default English prose policy is deliberately small:

| Rule | Level | Finding |
|---|---|---|
| `Cqlt.Repetition` | warning | Adjacent repeated words |
| `Cqlt.VagueClaims` | warning | Selected promotional phrases that need a concrete capability or evidence |
| `Cqlt.Wordiness` | warning | Selected phrases with a shorter ordinary alternative |
| `cqlt.text.nonempty` | error | Empty or whitespace-only text |

Warnings are review suggestions, not automatic rewrites or proof that a phrase
is wrong. Vale understands Markdown and excludes code by default. Quoted prose
may still need review. A clean report does not prove good framing, factual
accuracy, legal suitability or a complete public profile. Semantic review is
separate; see [the research assessment](../studies/semantic-quality.md).

## Library contract

1. Construct `prose::Plan` from explicit `Text { subject, format, text }` inputs.
2. Materialize `plan.files()` into a fresh private workspace. Generated paths
   are independent of subjects, so a subject cannot escape that directory.
3. Call `plan.run(execute)`. The closure runs argv in that workspace, returns
   stdout on success and propagates failures. Bound its time and captured output.
4. Retain the normalized report. For offline replay, retain the version and
   raw Vale response too and call pure `plan.report(version, response)`.

cqlt generates an isolated configuration and bundled style files. It runs the
installed `vale` with explicit configuration, no global configuration, JSON and
per-rule counts. It never runs `vale sync`, installs tools or edits inputs.
Vale 3.23.0 is exercised in CI; older versions without `--counts` fail visibly.

The report binds normalized inputs and policy with SHA-256, identifies the Vale
version and lists each document's content hash. Findings include subject, rule,
severity, matched text and source location. Sorting removes input and alert
ordering differences. Exact replay is deterministic; live equivalence also
requires the same Vale binary and runtime. The host is responsible for retaining
the executable's identity, as CI does with a verified release archive.

Missing executables, failed processes, malformed JSON, absent rules, unexpected
documents and inconsistent alert counts are execution errors, not successful
audits. Vale emits only files with alerts; a clean file is represented by cqlt's
explicit input manifest. Inline Vale suppression directives retain Vale's normal
semantics and are not evidence of independent editorial approval.

Tunables: callers choose Markdown or plain text for each document and an error,
warning or suggestion exit threshold. The default rule catalogue is versioned
with `cqlt-prose-v2`; changing it requires a ruleset update. Input is limited to
4,096 documents and 16 MiB of text per invocation; output is limited to 16 MiB.
Use separate explicit batches for larger collections.

`prose::descriptions(snapshot)` selects organization and repository descriptions
from validated, complete forge evidence. Missing descriptions remain empty inputs
and fail the nonempty rule. Existing snapshots contain document paths and sizes,
not README bodies: supply actual README text separately to audit its writing.

## Validation

Normal Rust tests cover ordering, hashes, malformed output, missing rules,
scope validation and error propagation. `cargo test --locked --test vale --
--ignored` exercises the real installed backend: clean prose, each warning rule,
empty descriptions, Markdown code exclusion and repeatable reports. CI stages
a release archive with a fixed SHA-256 before running this explicit test.
