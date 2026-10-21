# Nested borrowed closure receiver adapters

This checkpoint extends the integrated source `2f6e942d`. A borrowed captured
closure can be forwarded through multiple references using its complete receiver
Self, leaf Instance and requested call trait. No owner, body key, generated AST,
unchecked permit or LowIR representation is added. PA24 remains planned.

Every reference level applies the canonical blanket implementation's own
permission. A shared level requires shared call permission from its referent;
a mutable level can forward mutable or once invocation, but cannot supply
shared invocation. The leaf's strongest permission is checked only after the
whole chain. Existing trait selection and sealed source/template proof remain
required. Neither successful traversal nor the leaf identity grants permission.

One binder-aware match of the complete source Self supplies source-slot images
and repeated-slot equality debts. Prefix mapping indexes those images once,
chooses the lowest matching source slot deterministically and caches each new
prefix image. An existing exact image is reused; other images append slots.
Original, unused, obsolete receiver and later generic slots keep their identities.
Nested callable binders remain rigid. The iterative walks have explicit budgets;
there is no repeated source match or scan of prior prefixes at each level.

The wrapper retains the full physical reference chain in its execution Self.
It emits one ordinary typed dereference projection per level, then performs the
existing checked reborrow of the leaf and invokes its strongest checked source.
Normal formation, region, ownership, loan and destruction checks prove this CFG.
Cleanup owns only the outer reference and leaves the borrowed environment alive.
The exact source proof remains required in final assembly, including generic
specialization and declared preconditions.

`pa24/tests/nested-receiver-outcomes.json` declares 21 independently qualified
outcomes: thirteen accepted programs and eight rejections. All thirteen accepted
programs execute through the pinned CPPGM backend. They cover shared/mutable and
three-level chains, external and generic results, quantified callable input,
post-call environment access and Drop. Opposing cases retain intermediate-level
permission failures, once-only leaves, overlapping loans and reference escape.
The unchanged `reference-nested-boundary` fixture in the earlier 34-case matrix
is explicitly superseded as an acceptance; its previous boundary and independent
Rust qualification remain retained.

Three new private controls cover per-level permissions, physical prefix order,
original/unused/receiver slots, nested callable binders and repeated-image debts.
At prefix widths 8/32/128, exactly one slot is allocated per distinct new image
and reused occurrences allocate none. The full source controls at reference
depths 8/16/32 each retain one wrapper: canonical steps are 27, typed work is
81/129/225, and trait selections are four. These counters do not measure total
compiler instructions, allocation or wall time.

Evidence is retained in `obj/parallel/pa24/nested-receivers/`.
`hypotheses-v1.json` precedes implementation, and `qualification-v1.json` records
all rustc/self-check outcomes. `first.json` records fifteen entry/mapping/debt
invariants passing, followed by a test-only nonexistent-counter compilation
failure. `second.json` records the corrected three source/native/depth groups
passing. `third.json` records ten source/adapter proof controls, the updated
34-case adapter matrix with twenty accepted natives, and the 22-case generic
matrix with seventeen accepted natives passing. Its Clippy stop on the new
constant-size chunk iterator is preserved, as is `fourth.json`'s test-only array
reference comparison error after adopting `as_chunks`. `fifth.json` records the
three corrected private controls, strict release all-target/all-feature Clippy,
architecture audit (1446 files, zero findings), layout38 and pinned upstream
24+126 checks passing. Production changes were unchanged through those test
corrections. CPPGM tests are explicitly ignored by default and selected with:

```sh
python3 scripts/storage.py run -- cargo test --manifest-path dev/Cargo.toml --test environment_execution chains:: -- --include-ignored
```

Dynamic callable conversion remains the integrator's separate proof path.
This checkpoint changes ordinary captured invocation and does not activate PA24
or regenerate references.
