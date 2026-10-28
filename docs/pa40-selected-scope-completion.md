# Selected nested lint scopes

A checked function can contain an unchecked nested function. Its inherited
expectation must remain pending until that nested body can produce its events.
The previous selected-runtime path finalized the parent solely from its own
checked bit, producing a false unfulfilled warning. Baseline probe
`1793202494823255504` reproduces that failure. The two new PA40 source fixtures
have four matching pinned Rust observations across editions 2021 and 2024.

Completion now makes one paid join between pending expectation identities and
unfinished bodies' actual lexical policies. It runs only when a semantic scope
could otherwise close. Source declarations are visited once, with a fixed member
bound per unfinished body; no per-expectation declaration walk or ancestor search
is added. Unrelated bodies do not block a checked function's own expectation.
The same census supplies whole-source completion. No body is demanded, and no
persistent owner, node or body record grows. Missing lexical rows reject rather
than becoming absent policies. Temporary maps are paid before allocation;
failed queries retain attempted work without finalization.

The final joined gate `1793205123313052124` passes nine exact private controls,
ten lexer tests, PA6 and all 805 PA13 cases in138.84s, including both strict
Clippy modes, formatting and architecture checks. Its receipt SHA-256 is
`9636cd90a199d21fecd64320dae9e0e86c4123a8a34dff974de57f56dfb27d34`.
Controls include nested fulfillment/no-event, unrelated scopes, request
interleaving, header-only completion, failed payment, audited cold sizes and
fresh offset/comparison producers.

Cold and repeated completion queries pay identical work. With 1/8/32/128 parent
expectations and their unchecked children, paid steps are83/437/1661/6557.
One expectation with128/512 unrelated bodies pays4051/15955 steps. These are
linear work observations, not wall-time speedups; temporary storage is bounded
by pending identities and retained policy rows do not grow. The initial work
bound failed because it omitted paid empty member slots; the corrected bound
accounts for the u16 member capacity and still excludes quadratic owner/body
walks. Both the failed observation and successful measurements are retained.

Root's postcheck records the work observations, source/oracle joins, registration,
placement, upstream verification and one unchanged original-core observation:
`obj/parallel/combined-library-frontier-v1/observed-v2/receipt.json`, SHA-256
`b071255064b85793b90405149bd43686fb2886e1d23063461b89746853b99964`.
Fresh comparison and offset LowIR/source bytes and the pinned native backend
match the earlier native receipts exactly; those observations are reused without
claiming a new backend execution.

CLI `1f76fbd3f698b09a3bdcd327f6b6ba2a2cd7eee7b9869c7bfe0f08ec734f31a5`
still rejects original `num/float_parse.rs84` at the unsupported
`ineffective_unstable_trait_impl` producer. All722 source pins/configuration are
unchanged, with41 source opens. This is not a compiled percentage or an original
core, alloc or std artifact. PA31/PA40 and imported deprecation remain open.

The earlier combined build hit the unchanged16GiB cap on interrupted codegen
scratch. The qualified preflight now retires idle loose objects and known unlocked
abandoned sessions, preserving warm caches and pins; all266 harness tests pass.
About4.7GiB was reclaimed without changing the150GiB reserve or worker limits.
