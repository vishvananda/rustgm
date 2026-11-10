# Computed callable proof ownership

Three width128 reductions reproduce the full f01 CI regressions: independent
closure contexts, tuple contexts and repeated captured callable objects. Their
expected accepts were declared before independent Rust1.98 object checks and
text-oracle generation. Immutable baselinee8a01f0c rejects all three with an
inference work-budget error; the repaired compiler accepts them.

PA24 owns all three, including the captured object built on PA23. Computed
callable selection retains body and capture duties for its own callable and
final body-region consumers. It must not repeat an unrelated early closed-call
graph check. The ordinary PA21 static-only regression still rejects.

Existing width8/32/128 invariants retain their exact work limits and growth
checks. Independent contextual negative cases still reject escaping lifetimes
and invalid callable permissions. These fixtures add concrete manifest inputs;
they do not replace those scaling or negative controls.
