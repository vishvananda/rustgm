# Impl member identity

These eight original source contracts cover scoped macro identifiers in trait
implementations. The frozen hypotheses pass sixteen independent metadata/object
checks and four native programs with pinned Rust 1.98.0, edition 2024, x86-64
Linux, panic abort and overflow checks. `review.json` binds the inputs, tool,
profile and all 24 observations. No PA31 grading manifest is activated.

An impl identifier is normalized and adjusted against the actual trait container
before matching a member's namespace and kind. A macro-generated literal can
therefore implement an ordinary trait member. Two differently marked source
identifiers can map to the same canonical member, which is a duplicate even when
their original declaration keys differ. Methods, associated types and constants
obey this rule. Captured names from separate expansions retain their identity;
an unrelated opaque trait method does not become visible by matching spelling.

These are independent language contracts. General scoped invocation remains
unsupported by RustGM until its complete expansion and consumer gates pass.
