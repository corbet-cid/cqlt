# Semantic review protocol

`cqlt::semantic` builds versioned TypeSafe AI Jev requests and replays recorded
responses without I/O. A case supplies a stable `subject` and a `state` with
`audience`, `description`, and explicit `facts` (`purpose`, `kind`, `maturity`,
`capabilities`, `limitations`). Missing facts remain absent; the model cannot
infer them from a repository name or a passing deterministic check.

The `cqlt-semantic-v1` policy asks separate `purpose`, `claims`, and `maturity`
choice questions. The result retains each choice, confidence, full probability
distribution, model identity, usage, and SHA-256 hashes of the state, request,
response, and policy. `insufficient_evidence` yields `unknown`; other choices
yield `review`. There is no automatic pass, fail, score, or calibrated
threshold. A replay requires one response per case,
the exact request hash, all expected answers and the pinned `jev-1.13.0` model.
Malformed or incomplete responses are errors.

`Plan::requests()` gives the caller the serialized HTTP bodies. The caller owns
authorization, private input selection, transport limits, credentials, and
retention. Each state is capped at 16 KiB and a response at 1 MiB. Public
synthetic cases in [`experiments/editorial-cases.json`](../experiments/editorial-cases.json)
carry proposed labels, which must never be included in model requests. These
cases do not establish accuracy. Review against held-out labels is required
before a downstream policy can make automated decisions.

The HTTP shape is documented in the [TypeSafe API reference](https://docs.typesafe.ai/api),
and the [model page](https://docs.typesafe.ai/models) identifies the pinned
model and price. Fresh calls can disagree; only replay of a saved response is
deterministic.
