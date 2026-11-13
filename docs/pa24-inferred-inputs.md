# Inferred closure input lifetimes

The borrowed-input repair handles the original core pattern macro's
`(|&s| s)(self)` shape through the existing inference and body-proof machinery.
The original core has not been rerun with the successor compiler yet; candidate12
remains authoritative and no original library artifact exists.

## Contract

A parent's canonical lifetime and its existing free body alias enter a nested
input hole as one identity. Closing the result still rejects an input-only or
local lifetime that cannot escape.

For example, this generic original and its demanded concrete instances keep the
same reference depth and source lifetime:

```rust
fn relay<'b, 'c, T>(x: &'c &'b T) -> &'b T {
    (|&s| s)(x)
}
```

Noncapturing instance preparation matches complete typed signature roots. Used
inherited slots must match their actual captured arguments. Public late binders
stay rigid; private input ports connect to caller regions outside that binder.
Nested binders, repeated-region equalities and projection debts survive the
ordinary instance proof. A signature connection supplies identity, not a trait
premise or permission to discard a lifetime debt.

The public callable signature and the body's proof signature can differ even
without captures. The existing owner signature pair retains both, source Copy
checks authenticate the full closure instance/source/pair, and lowering checks
physical ABI agreement before demanding the body.

A noncapturing original also retains the creator's actual declared trait
premises with a child-owned lifetime-alias ledger, even when no expression
independently requested a trait goal. Concrete instances discard and reprove
that environment. A missing Copy bound still rejects. Captured execution keeps
its existing separate receiver scope; this context-retention extension does not
cover additional captured-receiver contracts.

Structural Copy/Clone marker components retain the completed child allocation
frontier. Sibling implementation proofs therefore receive distinct witnesses;
separate input lifetimes never become equal merely by selecting the same impl.

Reference coercion waits for a known terminal expected type before selecting an
extra dereference. With expected `&T` and actual `&&u8`, it can infer `T = &u8`.
The ordinary mutable/shared and lifetime proof checks still apply.

## Owning reductions and evidence

All outcomes were declared before independent oracle execution. Malformed
source1, ordinary4 and source6 setup attempts remain recorded; corrected inputs
were declared separately, with no expected positive inverted.

- PA20 `reference-tail-inference`:10 outcomes,7 accept/3 reject, all7 positives
  execute against Rust.
- PA24 `inherited-input-regions`:28 outcomes,19 accept/9 reject, all19 positives
  execute against Rust.
- PA31 `inherited-input-regions`:37 actual-language-item outcomes,27 accept/10
  reject, all originals pass their expected prepared-body outcome. Two actual
  Rust/C consumers exercise six exports through RustGM and the pinned backend.
  Their outputs are `13 29 0 13 13 1` and `13 13 13 29 29 29`.

Focused receipts1794586372014375955 and1794587214204194259 cover ordinary and
source originals, export preparation, source Copy/Clone, captured callbacks,
nested closures, reference coercion and specialization scaling. Native receipt
1794586605899163087 passes26 ordinary programs and both actual source consumers.
Receipt1794589096793583294 also passes the current source consumers and the existing captured-drop native control. Complete active course comparison passes all9345 cases; the separate released PA25 unit passes113/113.

The source marker scaling check independently accepts widths8/32/128 in Rust.
Inference work is641/1625/5561 and canonical work2329/6193/21649: both grow
linearly with distinct input lifetimes. Receipt1794587685884856582 and its
hash-checked replay retain the inputs, diagnostics and counters under
`obj/parallel/closure-return-inputs/`. Immutable calibrated timings remain due;
these work counts make no wall-clock speed claim.

Exactly38 new PA20/24 text triples were generated after native qualification and
reviewed against their declared outcomes. Every28,260 pre-existing sidecar is
unchanged. All9345 active comparisons and the separate113-case PA25 unit pass.
The315 Python harness tests,486-root registration,44-stage/component layout and
upstream verification pass.

The three new private opponents retain unpaid lifetime debt, oppose changed
public/inherited slots, and oppose changed source Copy instance/signature/
environment identity. Their execution remains pending: cold private lint/link
attempts, including a quiet267s attempt, hit the bounded validation limit without
producing a harness. Scheduled
full qualification remains required. This checkpoint does not release PA31 or
complete any original library artifact.
