# Associated Copy values at cleanup roots

Clean release25 clears the Unicode initializer-storage failure in unchanged
original core. Its next completion error is ptr/metadata.rs119's original
`from_raw_parts`: active cleanup requires const Destruct in its defining
environment. All42310 declarations,25858 bodies and35120 item attempts still
pass, with no failed or blocked functions; no original library artifact exists.

The reductions isolate an owned associated-type parameter whose declaration or
where clause already requires actual source `Copy`. Ordinary controls pass;
source declared/where/moved cases reproduce the core diagnostic. The existing
qualification code can prove Copy, but previously skipped bare projected roots.
After admitting those roots, a diagnostic trace confirms successful scoped Copy
selection for both source forms. The remaining root mask always kept full-type
cleanup active even after that actual NoDrop answer.

Root projected values now enter the existing qualification partition. Root masks
honor actual NoDrop answers, including genuine scoped Copy selection. A Destruct
qualification keeps the root's separate cleanup requirement; an actual own Drop
requirement is never filtered. No Copy/Destruct rule, declaration-name exception,
body/frame field or backend contract is added. Selection still retains its exact
source environment, type applications and obligations, followed by the existing
region/owner checks. The initializer-storage, constant-call and borrow guards
remain independent.

The PA25/PA31 `projected-copy-cleanup` manifests freeze16 inputs before independent
execution. All independent Rust outcomes agree with declaration, and accepting
inputs execute with declared values. The qualified subset is13 outcomes:
ten accepting, three reached unpaid/lookalike rejections. Five source programs
produce real nonempty LowIR and native7; ordinary controls complete ANSWER7.
Three additional unused-definition negative inputs still pass the candidate's
analysis-only view without a reached call. Their required rejections, unchanged
bytes and original observations remain explicit; they are never inverted or
silently replaced by the separately added reached opponents. Whole assignments
stay planned.

Final fast1794656862666427149 passes six exact tests across frontend/traits,
strict selected/CLI Clippy, formatting and architecture. Final native
1794657051948254620 passes three groups/eighteen actual programs: five new source
observers and thirteen old real cleanup-effect observers. Actual destructors,
conditional flags, move/loan/unsafe faults and expired-storage guards remain
checked. A temporary typed-body inspection test was removed after preserving its
source and output under `obj/parallel/projected-cleanup/`.

Authenticated replay binds all3962 compiler/test sources, every packet input and
frontend harness SHA
`78bfda31ede60b5117899f300292738627fd3dbb215554b4a60a78413aba07a3`.
Widths8/32/128 retain inference/canonical/CTFE step tuples
`424/2507/88`, `1048/2651/256`, `3544/3227/928`. Both ordinary Copy controls and
source parameter/reference controls pass. All315 harness checks,494 roots in13
binaries,44 assignment layouts, the separate component,128 upstream fixtures and
126 Reference checksums pass. No timing speed claim is made.

The previous full43588626797 on exactff072ae85 passes every stage, including3078
private tests in release and debug. This new compiler checkpoint still requires
its full successor and a fresh unchanged original-core observation. Prefer a
cached dev-fast diagnostic with matching source cfg, keeping120s and every quota
unchanged; a timeout remains a failure. Metadata/LowIR and original library native
consumers require successful original analysis first. The old pointer-composition
provider's weaker Metadata:Sized declaration and original wrapper bytes remain
unchanged; no prior positive, callback artifact or transport requirement is closed
by this finite checkpoint.
