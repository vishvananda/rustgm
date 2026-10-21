# PA26 operator and shared-index checkpoint

This is an implementation checkpoint, not a PA26 release. PA26 stays planned;
its declared draft source outcomes are in `pa26/tests/operator-outcomes.json`.
No generated text reference or assignment activation is included.

## Typed boundary

Demand-loaded library metadata owns distinct trait, Self/RHS, method and Output
identities for Neg, Not, Add, Sub, Mul, Div, Rem, BitAnd, BitOr, BitXor, Shl, Shr
and shared Index. Ordinary resolution handles aliases and shadowing. Metadata
methods and associated types enter the existing conformance inventory through
canonical declaration identities; their signatures are typed data without a
fabricated declaration AST.

Body inference retains the complete trait goal and projected output. The
existing selected method-call facts and dispatcher carry the actual callable
into normal Call lowering. Shared Index creates its receiver loan before its
index expression and materializes its returned reference before projecting a
place. Source expression evaluation, ownership and lifetime validation continue
through the ordinary body representation.

## Qualification

All commands used the shared resource controller, two Cargo/test jobs, the
isolated worktree `obj/`, and its configured pinned external backend.

- `cargo check --manifest-path dev/Cargo.toml --locked`: pass.
- `cargo fmt --manifest-path dev/Cargo.toml --all`: pass.
- `cargo clippy --manifest-path dev/Cargo.toml --all-targets --all-features --locked -- -D warnings`: pass.
- `language_operators`: 11 declared source outcomes and five independent native
  comparisons pass. The native test was then marked ignored, matching the existing
  maintainer-native convention; repeat it with `--include-ignored`.
- Independent pinned-rustc object generation and RustGM CLI lowering agree on all
  11 fixtures. Diagnostics were reviewed for the actual shared loan, shared write,
  moved read, method result and mismatched RHS failures.
- Focused earlier suites: `associated_type_declarations` 3 pass/1 ignored,
  `call_operators` 1 pass/1 ignored, `call_trait_contracts` 4 pass, and
  `trait_bounds` 116 pass/10 ignored.
- Rust syntax/architecture/file audit: 1,157 files, zero findings.
- Assignment layout: 38 stages; pass. `git diff --check`: pass.

The object qualification receipt and retained tool diagnostics are under
`obj/parallel/pa26/operators/`. Resource command receipts are under
`obj/parallel/resources/`; strict all-target Clippy is receipt
`792a9776970d4b1ab424e8fd850904d8.json`, the earlier focused regression batch is
`9dc40da86df04002b69b6b1fe0b2d4f9.json`, and the file audit is
`b06c6c24889243d4b9c1986bb78136ba.json`. Generated executables and objects were
pruned after successful qualification; these receipts retain evidence rather
than an assertion that old binaries remain available.

## Remaining work

Mutable Index, assignment/comparison/dereference hooks, built-in primitive
operator trait implementations, iteration, question-mark, additional derives,
formatting and nominal Option/Result/ManuallyDrop metadata are not part of this
checkpoint. Library method syntax beyond the checked operator entry points still
needs its own source/native qualification. Full release/native regression,
all-active text comparisons and controlled performance measurements remain
integration/release work; no release readiness is claimed here.
