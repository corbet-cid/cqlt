# Editorial judgment experiment

`editorial-cases.json` contains synthetic cases and proposed human labels.
`jev-request.json` is an offline request template with one concrete synthetic
case and a versioned Jev model. No responses or benchmark results are included.
`semantic-cases.json` contains the eight case states in the exact input shape
accepted by `ccid quality semantic`; it excludes the proposed labels. Running
that command without `--live` prints request hashes and sizes without network
access. A live run still requires explicit authorization, credentials and caps.

For an authorized experiment, substitute each case's `state` into the template.
Keep expected labels out of requests. Preserve the exact request, raw response,
model identity and content hashes. Compare per-question errors and abstentions;
use a separate reviewed held-out set before tuning production thresholds.
Do not use these eight cases as evidence of calibration or accuracy.

The request files do not execute requests, read credentials or spend money.
See [the semantic quality assessment](../studies/semantic-quality.md).
