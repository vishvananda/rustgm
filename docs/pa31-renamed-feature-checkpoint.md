# Renamed library-feature source qualification

The compiler now accepts actual old_name declarations and rejects former-name
requests as hard errors. Former names do not enable APIs. This clears LIB-127
and preserves the remaining authenticated relationship-metadata and original
library-artifact requirements; PA31 remains planned.

The [independent packet](../pa31/tests/spec/renamed-features/README.md) binds Rust
1.98.0 attribute parsing, feature collection and request checking. It qualifies
44 programs, including three fresh rustc consumers of an actual rlib, source-core
providers, const policies, unrestricted old-name strings, first-declaration and
canonical-name collisions, malformed attributes and release-version conflicts.
It does not claim RustGM can publish that aliased provider yet. Old metadata
profiles explicitly reject relationships they cannot encode.

Implied-by and former-name links share the existing cold vector with a typed kind
and exact policy ID. Policy parsing prepays both links before publication. One
transient borrowed feature index retains the first unstable declaration and
checks stability/release consistency; another request index exists only when
former names need checking. No request scans all declarations, and semantic uses
do not repeat this work. The paid 16/64/256 declaration/request scaling invariant
passes. Existing per-policy const eligibility and implication rules remain intact.
Current release uses the fixed Rust 1.98.0 profile rather than the host compiler.

Final compiler/test source evidence:

- 11 exact private controls, format, architecture and strict Clippy, 220.98s:
  obj/qualification/fast/1793749359877317133/receipt.json, SHA-256
  `72683040f71829946500e24d915a6895d3cb24dd2e9969d009f3513e4aa77b1f`.
- Production smoke plus complete feature_requests and availability_contracts
  ordinary lanes: 27 exact tests across two targets, 206.22s:
  obj/qualification/fast/1793749607265771712/receipt.json, SHA-256
  `f84727d0865fd75fee64401d0f3b8187ce2256d41a49afed824994b948a08291`.
  One selected availability test is ignored and remains in full native CI.
- Independent pinned Rust, 44 outcomes including actual imported consumers and
  native7, 1.26s: obj/parallel/renamed-features-v1/oracle-v4/receipt.json,
  SHA-256 `d09a5867b51692c703955788986b225290b96912014eb3bd0e1f649a7ecc424f`.
- Joined qualification, 37.04s: 82 CLI phase outcomes over41 local/source-provider
  programs, previous-candidate rejection, actual native7 from the real RustGM
  lowerer and locked CPPGM object backend, upstream and harness checks, and the
  unchanged original core probe. Receipt:
  obj/parallel/renamed-features-v1/qualification-v1/receipt.json, SHA-256
  `b799f069ba78a69c2e1d612343ef6af0ab0156929230a1b9d1bac114f83feb07`.

Immutable CLI:
obj/parallel/renamed-features-v1/cli/fc5d8d123b666903ab3ca3b88cf8de55a149483944e5786d2b975208b1046c24/rustgm.
Actual unchanged sizes include Library152, Inventory112, Policy56, Package256,
Body456 and NodeFacts80. No AST/body permission field or persistent alias map was
added. No wall-time speedup is claimed. Failed reference setups, the initial
Clippy findings and the interrupted refactor check remain retained. No outcome
was changed to hide a failure; the native observer received its real public
stability annotation and imported clients received distinct crate identities.
The previous a83ea4a15 full CI41739306105 passes every gate.

Original core clears old_name on ops/mod.rs197 and reaches LIB-128:
rustc_default_body_unstable(feature="pin_ergonomics",issue="130494") on
Drop::drop's provided body, library/core/src/ops/drop.rs242. The current error
is file90 byte9919, const/default-body/macro stability contracts unsupported.
Observation takes19.9603s/375source opens, with all722 source hashes, six
configuration hashes, target/options/capAllow and finite limits unchanged.
Result SHA-256:
`24f76b479b5291b431d06151db797a329077b67d87eed35e6d1c0b75261aedb5`.

Next implement actual default-body policy and inheritance/use permission. A stable
trait API with an unstable provided body must not become a no-op, enable the
unstable body globally or make every explicit override unstable. The selected
implementation/body, destructor semantics, constant contexts and imported policy
must retain real ownership. General nominal/value/relationship transport and
original core, alloc and std artifacts remain required.
