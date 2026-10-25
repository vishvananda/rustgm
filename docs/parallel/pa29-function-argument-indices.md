# Function argument indices after lifetime declaration

CI40209517783 completed the missing private build and all ten exact cases in
79.79 seconds after the mandatory42.20-second CLI build. Nine passed. The
remaining anonymous-function opponent reached a genuine declaration mismatch:
`borrower<'a,U>` retained U at canonical index1 even though its actual type/const
argument vector contains U at position0. The earlier40205054394 build timeout
and40198294955 seven-pass/three-failure receipts remain separate evidence.

`declarations::generics::declaration_parameters` enumerated syntax nodes before
skipping function lifetime declarations. The corrected producer removes those
validated lifetime nodes before assigning the type/const argument positions.
Function lifetimes continue through the existing signature first-use and
early/late classification. Aliases, aggregates, traits and impls keep their
existing mixed lifetime/type/const argument order. No schema or profile changes
are made; malformed source slot indices still fail the existing strict check.

The owning PA18 regression invokes actual `borrower<'a,U>`, both type/const
orders and an ordinary mixed-lifetime alias. It checks genuine parameter owner,
kind and argument position, with an independent Rust metadata acceptance check.
The existing two-lifetime function control deliberately changes its old T index2
expectation to0 and adds N index1; its late slot order stays[0,1]. The PA29
regional-substitution opponent now checks those genuine local and transported
indices, corrupts a real slot index to ensure rejection, and retains the actual
caller-region substitution opponent.

## Expected text effect before reference generation

For a function with L written lifetime parameters and a genuine type/const
argument at position j, its Parameter.index changes from L+j to j. Its owner,
kind, actual argument order, signature region allocation and callable scheme stay
unchanged. Nonfunction mixed argument indices stay unchanged. Type and const
semantic views therefore change their printed indices. The scoped publication
below uses actual compiler output, without replacing numbers in references.

Concrete existing examples are PA19 conformance-trait-equal-lifetimes (actual
function owners17 and19: type index2 becomes0, with binders2 unchanged) and
PA20 callable-projections/semantics/borrowed-projection (apply owner20: type
index1 becomes0). PA18 accept-reference emits only main, so its LowIR should stay
unchanged. The source-only candidate inventory under
`obj/parallel/pa29-source-client-staging/lib011/function-argument-indices-v1/expected-text-effects.json`
lists211 explicit positive-index parameter rows across171 reference files.
Those are triage candidates: actual typed owner/formal facts must decide any
change, especially const-first declarations. It is not an oracle rewrite list.

The new PA18 fixture has its success/native contract declared in the owning
manifest. Actual independent Rust metadata and the three selected declaration
controls pass in CI40211701382; native execution remains a separate release check.

The immutable candidate and baseline execute all 8485 case pairs in 79.86 seconds.
There are 8362 exact pairs, 122 existing stdout-only index changes and one new
accepted fixture. Every old status, diagnostic and all 4902 old LowIR outputs
are unchanged. The new fixture's LowIR also equals the baseline. The old 25452
reference channels exactly match the baseline observations.

Configured AST/resolver diagnostic views authenticate all 151 changed parameter
rows against 140 actual function owners, their active declared lifetime counts
and actual type/const formal positions. All 211 nonfunction parameter rows in
those changed cases remain exact. One genuine anonymous input parameter is
joined to its real resolver-reserved opaque input node; it is not fabricated.
Wrong diagnostic-mode and explicit-only reviewer failures remain retained.

The root reviewer rechecks those raw typed joins and all frozen hashes before
the existing course writer generates 123 selected cases: one new PA18 case,
35 PA19, 51 PA20 and 36 PA21 cases. Actual outputs must match the reviewed hashes
before each stage writes. Generation, all 8485 resulting reference-channel
hash checks, metadata, harness and upstream checks pass in 18.62 seconds.
Four scoped provenance receipts record the actual candidate's source tree.
This is fresh execution of 123 selected cases plus authenticated retained
observations for the other 8362, not another full compiler run.

The frozen review manifest is
`obj/parallel/anonymous-text-effects-8191/final-review/manifest.json`, SHA-256
`f6da951751d51d94cf74a529584d06a2abd19fa5742862d6e0233d212fa754c8`.
The checked writer and receipt are under
`obj/parallel/anonymous-reference-publication-v1` in the A worktree.

## Frozen focused selection

Private: the anonymous context/binder opponent, existing Functions codec/fresh
client control and existing malformed-slot rejection. Declaration group: the
corrected lifetime-binding invariant, new actual-call/formal ordering control
and existing complete alias substitution control. These cases pass in the two
focused gates above. Broad private/native qualification remains periodic CI work.
