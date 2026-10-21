# Generic captured receiver adapters

Captured closures inside generic owners can use the existing owned and
one-reference receiver adapters. Selection retains the complete closure
Instance, exact receiver Self and requested call trait. No new body owner,
source identity, unchecked permit or syntax representation is introduced.

The adapter demand first materializes the exact source through its checked
template and proves that specialized body. Only its sealed proof can authorize
the derived wrapper. The wrapper retains the original semantic source and
checked template linkage, while its private Template validates the full
specialization, public and execution signatures, and receiver. Ordinary region,
ownership, loan and destruction checks then prove the wrapper independently.
The source proof remains in final assembly. Parent conditional conclusions and
source-local assumptions are not copied into wrapper authority.

Argument substitution preserves the complete source execution binder. Self
normalization retains separate debts before requested receiver shaping;
requested signature normalization retains its own debts afterward. Both are
rebased and checked. An owned FnOnce adapter leaves the old hidden receiver
slot unused, so later generic-argument slots keep their identities. Borrowed
receiver prefixes are distinct from source/environment slots, and nested
callable binders remain rigid. Repeated slot matches emit equality obligations.
The ordinary direct execution helper composes its existing normalization path.

One transient full-Instance index replaces the prior demand visit set and
records sealed body positions, avoiding source scans or duplicate
specialization before wrapper proof. This stores an index payload for ordinary
generic instances too; the change does not claim zero memory or instruction
cost. Template-free ordinary programs retain the existing early path. The
existing instance/body budgets apply.

`pa24/tests/generic-receiver-outcomes.json` retains twenty-two independently
qualified Rust outcomes: seventeen accepted programs and five language
rejections. Thirteen accepted programs currently execute through CPPGM,
covering owned/reference Fn and FnMut forwarding, cleanup, const arrays,
unused arguments, later argument lifetimes, quantified inputs and nested
callable binders. The negative cases retain move, loan, permission, escape and
output obligations. Four Rust-valid cases remain explicit follow-ups: two
generic external-reference results through a callable helper, and two selected
projected outputs. Their original acceptance hypotheses and failed frontend
observations remain preserved; they are not Rust language rejections.

Private controls reject missing checked source conditions, forged source
execution, the wrong specialization/source index, and an unpaid private-borrow
outlives obligation. A paired exact source succeeds. A slot invariant verifies
the unused receiver slot, later generic slot, nested binder and repeated-slot
equality together. At widths 8/32/128, repeated requests retain one adapter and
67 canonical steps; distinct requests retain 8/32/128 adapters and
347/1307/5147 steps. These counters measure canonical work, not total compiler
instructions or memory.

The new source/native/cache groups, specialized-source invariant and existing
receiver source/native groups pass. Strict all-target/all-feature release
Clippy, architecture audit (1423 files, zero findings), layout and pinned
upstream checks pass. Receipts and earlier failures are retained under
`obj/parallel/pa24/generic-receivers/`; `fifth.json` records functional checks
and `sixth.json` records the final lint/audits. `seventh.json` rechecks source
outcomes and cache behavior after the source-lookup helper refactoring.
Native tests are explicitly
ignored by default and selected with `--include-ignored`:

```sh
python3 scripts/storage.py run -- cargo test --manifest-path dev/Cargo.toml --test environment_execution hierarchy:: -- --include-ignored
```

This checkpoint does not complete PA24 or activate its assignment. Nested
reference adapter chains and dynamic callable objects remain separate work,
along with the four retained accepted cases above.
