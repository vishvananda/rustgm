# Complete original coroutine module

`original.rs` is the complete pinned Rust 1.98.0 core ops/coroutine.rs, unchanged.
Every complete provider embeds those exact bytes, including all eight derives
and both real forwarding implementations. Its provenance and whole-provider
companion are registered in third_party/sources.json.

The dependency foundation reuses the existing const-parameter source fixture and
adds authored Pin, reference, comparison, Hash and formatter declarations. Its
formatter/comparison helpers support type checking; they do not implement or
claim the complete standard formatting/hashing runtime. Actual original coroutine
bodies and the ordinary user implementation remain intact and execute normally.

Nine independent predictions are frozen in predictions.json and cases.tsv.
They include both real forwarders, a valid pinned !Unpin coroutine, the invalid
&mut G forwarder for that !Unpin type, a valid returned borrow, an escaping borrow,
wrong Yield/input types and an unrelated same-named method owner. Pinned Rust
accepts four and rejects five with actual E0277/E0515/E0308 causes. The literal
C observer executes both provided forwarders and expects `23 47`.

The native reference uses panic=abort and overflow-checks=off, so the dependency
foundation's unused primitive Neg implementation does not need an invented panic
hook. The two observed coroutine executions perform no overflowing arithmetic.
This flag is explicit in the reference receipt; no panic/runtime parity is claimed.
The whole-provider metadata companion also passes under the independent checker’s
ordinary upstream_check crate name and flags.

Diagnostic-only PartialEq/PartialOrd hints are separate controls: pinned Rust accepts their
ordinary declarations and rejects using them as operator language roles. They
grant no implicit operator, caller, move or body permission.

Candidate results are separate from these reference predictions. Initial candidate
observations exposed over-eager diagnostic/language coupling and missing generic
factory result context; see the maintainer checkpoint/evidence for their status.
No fixture is a generated-coroutine, whole-assignment or original-library artifact
completion claim. PA31 owns the original identities/bodies, PA26 the derives,
PA18 the generic context, PA42 generated state and PA43 capture/cleanup.

The separately frozen result-coercion control keeps an explicitly written
Pin<&[u8;2]> input and its genuine CoerceUnsized result to Pin<&[u8]>. A generic
factory hint must not rewrite that fixed input to the expected output type.
