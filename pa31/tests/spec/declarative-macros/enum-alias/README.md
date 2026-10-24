# Contextual enum aliases (LIB-002)

These PA31 hypotheses isolate the actual variant suffix of a type-alias path.
They require scoped declarative macros and remain outside active grading suites.
The contract is declared before independent execution; do not rewrite a failed
hypothesis to match an observation.

`captured.rs` supplies owner, alias and variant names from the caller and should
print `7`. `introduced.rs` captures the owner and alias while introducing the
variant and its matching use; it should also print `7`. A captured container
name does not make its introduced member a physical identifier.

`caller-opponent.rs` attempts to name that introduced variant from the physical
caller. `suffix-opponent.rs` opposes a captured declaration with a literal suffix
inside the expansion. Both are predicted language rejections. All four sources
are checked as metadata and objects with pinned rustc; the two positive programs
also require native execution. An earlier unsupported-macro error in RustGM is
not evidence for either negative. Scoped source admission remains disabled.

The corresponding semantic consumer must use the suffix's retained context,
the actual enum container introduction, the contextual binding index and the
adjusted definition-site privacy. Do not derive these from the alias or whole
path's first segment. Field/method/associated-type consumers remain separate
LIB-002 requirements.

The independent run subsequently matched all eight language observations and
both native programs. `review.json` binds the exact source/contract and oracle
receipts. The maintainer's semantic-consumer qualification is recorded in
[the focused checkpoint](../../../../../docs/pa31-enum-alias-lookup.md);
these observations do not activate scoped source admission or PA31.
