# Semantic quality: Jev and related approaches

Research assessment, 2026-09-28. No model requests or measured quality results
are claimed here. Vale is implemented; the semantic options below are research.

## Division of responsibility

| Technology | Useful role | Boundary |
|---|---|---|
| Vale | Terminology and writing conventions using local configurable rules | Cannot establish whether a project's claims are true |
| TypeSafe AI Jev | Narrow, typed judgments about supplied text and evidence | Hosted model; probabilistic judgments need evaluation |
| jev-lint | Existing matcher-plus-judgment tool for comments, names, tests and documents | Sends matched content to Jev; adds Node and API requirements |
| General-purpose model review | Explain problems and draft evidence-supported rewrites | Higher-level review, with model outputs retained as evidence |

Jev accepts a state and typed questions. Choice selects among declared outcomes;
Score evaluates declared levels; Noul returns a true/false probability. It does
not generate replacement prose or explanations. Its System One design keeps
workflow decisions in ordinary code. Calibration describes prediction groups,
not a guarantee for any individual verdict. [System One](https://docs.typesafe.ai/concepts/system-one),
[coding-agent integration](https://docs.typesafe.ai/introduction/coding-agents).

Choice/Score confidence summarizes the returned probability distribution. It is
not another independent verification signal. Preserve the distribution and test
thresholds against labeled examples from the intended task. Noul does not have a
separate confidence field. [Confidence](https://docs.typesafe.ai/confidence).

The documented version is `jev-1.13.0`; avoid moving aliases when comparing runs.
The listed input price is $0.042 per million tokens, with output tokens free.
This is a dated vendor price, not an authorized budget or measured operating
cost. Requests use a hosted HTTP API. [Models](https://docs.typesafe.ai/models).

The vendor documents weaknesses with counting, arithmetic, dates, indirection,
distracting context and adversarial input. Keep mechanical checks in Rust/Vale;
send only the evidence relevant to each judgment. Treat repository text as data,
including text that attempts to dictate its own verdict.
[Known limitations](https://docs.typesafe.ai/model-jaggedness/jev-1.13).

## Proposed quality questions

Each question should produce an individual finding instead of one overall score:

- Does the description identify a concrete purpose for the declared audience?
- Are its capability claims supported, contradicted, or unresolved by the supplied
  evidence? Missing evidence must remain unresolved.
- Does the described maturity agree with the supplied project status?
- Do descriptions across forges and README introductions describe the same
  capabilities and ownership, allowing different wording?

The [synthetic evaluation cases](../experiments/editorial-cases.json) exercise
clear/unclear wording, supported/contradicted/unknown claims, maturity drift and
an instruction-injection attempt. Their expected outcomes are proposed human
labels, not model observations or demonstrated accuracy. The matching request
template is suitable for an explicitly authorized experiment, not production.

## Evaluation before adoption

Use a held-out, reviewed sample across organizations, libraries, applications,
forks and archives. Compare the same cases against a normal model reviewer and
the deterministic baseline. Measure false positives, missed contradictions,
abstention, per-question calibration, repeated-run disagreement, latency and
actual cost. Keep prompt and model selection separate from the held-out labels.
Do not set a production threshold from these few synthetic examples.

Retain content and evidence hashes, exact question definitions, resolved model
identity, full responses and the policy that interpreted them. Replaying those
recorded responses can be deterministic; fresh model calls are a separate step.
Failing evidence collection or API execution must never become a clean report.

[jev-lint](https://github.com/mizchi/jev-lint) already combines ast-grep selection
with Jev judgments and provides dry runs, recorded-run replay and per-rule
evaluation. Assess reuse before building code/comment checks in cqlt. It is less
directly suited to forge metadata than narrow questions over structured project
facts. Its own [measurement report](https://github.com/mizchi/jev-lint/blob/main/docs/deepdive.md)
distinguishes contradictions visible in the supplied material from API-specific
defects better left to conventional tools. These are upstream observations,
not results from a cqlt experiment.

Recommendation: retain local deterministic checks as the default, evaluate Jev
as an optional subordinate semantic backend, and initially expose judgments for
review. Any hosted experiment needs an explicit budget and approved data scope.
No API credential, provider registration or hosted inference is required by cqlt.
