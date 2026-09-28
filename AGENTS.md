# Working on cqlt

This is a public, provider-independent quality library. Keep credentials,
operator inventories and organization-specific policies outside this repository.
Evaluation must remain pure: no network, clock, filesystem, subprocess or model
calls. Backends may request execution through an explicit caller-supplied runner;
cqlt owns their protocol and policy, and the caller owns bounded execution.
Every rule change changes the relevant ruleset identity. Missing evidence is never
a pass. Test ordering, incomplete evidence and policy mistakes explicitly.
