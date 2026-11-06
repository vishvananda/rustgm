# Source Field raw-autoref lexical policy

These exact source bytes were independently frozen before implementation.
They are relocated from the initial PA31 scratch packet to owning PA40 without
changing source hashes or outcomes; predictions.json retains that provenance.
The controls distinguish lexical allow/warn/deny/forbid, fulfilled/unfulfilled
expectations, explicit references, direct raw fields and an Allow cap. The cap
uses the unchanged PA31 raw-explicit input with its real compilation option.

The diagnostic consumes an actual generated borrow of a raw-pointer place.
It grants no unsafe, trait, borrow, call or const permission. The separate
rustc_no_implicit_autorefs source contract remains independently authenticated
for method syntax. Broader diagnostic inventory and library artifacts remain
required; this packet introduces one compilation-required deny-default lint.
