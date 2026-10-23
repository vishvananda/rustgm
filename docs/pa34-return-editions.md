# Return opaque capture editions

This local development checkpoint implements implicit return-position `impl Trait`
capture differences between Rust 2021 and 2024. It does not enable the public 2021
driver profile, implement external macro transport or activate PA34. Combined
release and immutable-binary performance qualification remain pending.

Free and inherent functions in 2021 capture only lifetimes appearing in the opaque
bounds. Trait declarations and trait implementations capture their input lifetimes
in both editions. Type/const captures and explicit `use<...>` selection retain
their existing contracts. A where clause alone does not capture a lifetime.
The `impl` token's defining edition controls this rule, including captured macro
fragments. Never-type fallback instead uses the compilation unit's edition.

The implementation retains resolved lifetime identities in the existing bound
syntax walk. It selects those belonging to the defining signature, so quantified
local lifetimes do not become outer captures. This inventory precedes alias
normalization: `Extra<Erased<'a>>` still captures `'a` when `Erased<'a>` resolves to
`u8`. Associated equality bounds have the same requirement. No extra whole-tree
walk or source reparse is introduced. Only legacy opaque continuations allocate
the optional inventory; ordinary owner, type and CFG records do not grow.

The primary references are the pinned Rust Reference's `types/impl-trait.md`
automatic capture edition rule and pinned rustc's
`rustc_hir_analysis/src/collect/resolve_bound_vars.rs`, specifically
`opaque_captures_all_in_scope_lifetimes`. The frozen rustc executable SHA-256 is
`3690cc576ede140504698405d5d8fa3826aaadbe71699c6c4ed0a565d6f493e2`.

## Independent and candidate evidence

`pa34/tests/opaque-edition-outcomes.json` declares sixteen local programs in both
editions and eight provider/consumer macro combinations before candidate testing.
All 40 source hypotheses match pinned rustc, and all 28 accepted programs execute
with status zero and empty output. The independent run retains 72 observations,
including four macro-provider compilations. Its results SHA-256 is
`07d19f39122ffc89127543f8db54a02745a9e9e4489f3acdec517c20fbcbee4a`.

Candidate checks use the ordinary package, semantic proof and lowering paths.
They cover all 32 local outcomes and eight token-origin combinations. The macro
tests redirect only definition-origin bytes to a donor unit; invocation fragments
retain their original source. All 28 accepted candidate programs execute through
the configured CPPGM backend with zero exit and empty output.

The successful focused batch also runs the existing never fallback, opaque
inference and opaque outlives controls: nineteen tests across three binaries.
Formatting, strict all-target/all-feature Clippy, the 2,080-file architecture
audit, 164 harness tests, registration and upstream checks pass. Evidence is in
`obj/parallel/pa34-opaque-editions/development-v4`; checks SHA-256:
`2a26995c309b21935fbab12702e206402aa3aee0e6f32147a4fce30db326ee06`.
The focused debug build/run took 92.17 seconds, followed by 4.63 seconds for the
native batch using its existing artifacts. These are shared-machine observations,
not an A/B performance claim. Scoped cleanup removed all 28 generated executables.

Earlier attempts remain available. Development v1 stopped at the filename-stem
audit, before compilation; the test is now `return_editions.rs`. V2 passed all
static checks and the existing neighboring suites but exposed two missing alias
captures: source lifetime parameters use the lifetime namespace rather than the
builtin lifetime definition kind. That classification is corrected. Two other
assertions rejected the correct live-loan diagnostic; the assertions now recognize
it. V3 was deliberately interrupted during audit after review found a prohibited
`expect_err` in the new test. The corrected v4 passes without lint suppression,
weakened semantic outcomes or changed independent fixture bytes. Documentation
was added after that source snapshot; it does not constitute another compiler run.
