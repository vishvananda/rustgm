# Capture-time library-feature indexing

The frontend now builds one feature index while capturing actual policies. Each
policy retains its own source owner, kind, release and relationship IDs, while
equal decoded feature spellings share immutable string storage. Completion reuses
the finished index instead of rebuilding it over all policies. Stable/unstable
and release conflicts still report the first mismatch at the original completion
boundary, after every source attribute has validated. No API, const, default-body
or metadata authority comes from a shared spelling.

Temporary argument strings borrow the parsed source or consume the actual decoded
escape buffer. Since/issue values do not need separate retained string allocations.
Capture prepays lookups, unique validation/copy/insertion, map growth and rehash
work from the unique retained spelling bytes. Failed payment publishes neither a
policy nor a canonical entry. The cold index is dropped at publication;
Library152, Inventory112, Policy56, syntax Node36 and Body456 remain unchanged.
The transient Draft grows160→208 bytes. Its extra map scales with distinct names,
not all policies. No quadratic baseline or improved asymptotic bound is claimed. Hash-table
growth still rehashes retained key bytes, with that work paid explicitly.

The [frozen contract](../pa31/tests/spec/stability-symbols/README.md) qualifies
eight pinned Rust outcomes, including deterministic16/64/256/4096-function crates.
The4096-function source exactly matches the independent reference bytes and now
completes full type analysis with606746 charged inventory steps under the unchanged
1000000-work/64MiB limits. The predecessor rejects that valid Rust program at its
inventory budget. Shared and distinct-name scaling, escaped-equivalent names,
separate const/default owners, deferred conflicts and failed allocation checks pass.

Final compiler/test snapshot evidence:

- 25 exact private tests, format, architecture and strict Clippy,145.77s:
  `obj/qualification/fast/1793755016504212126/receipt.json`, SHA-256
  `7901e0531889f082cbb06237fbc8fda0cc00a5eb048dbab6d1d5eac28d2f6317`.
- Production smoke and full feature/availability ordinary selections:27 exact
  tests,172.11s. `obj/qualification/fast/1793755179250903319/receipt.json`, SHA-256
  `381d0532a3f00340df38dc74eb65b12c64067196362b0e15b8e069058cf641e0`.
- Independent Rust8 outcomes,0.53s:
  `obj/parallel/stability-symbols-v1/reference-v1/receipt.json`, SHA-256
  `86008c0bcd6b7c5a042a09db0581c82f674bf6379725ffcedb82cb702a02192b`.
- Joined candidate qualification,41.33s:274 CLI phase outcomes,272 output pairs
  byte-for-byte equal to immutable predecessor17eb46c0, the two expected old
  large-crate budget rejections, actual native7 and7 9 from the real lowerer and
  locked CPPGM backend, upstream/harness checks and original core. Existing frozen
  default/rename/language packets retain their independent observations; all258
  current candidate phase views are rerun and paired. Receipt
  `obj/parallel/stability-symbols-v1/qualification-v1/receipt.json`, SHA-256
  `af8e16c29109b22f82f7107ad305bfbc22089dd9e2ea6922c389d57c2b8af861`.
- Immutable-binary wall-time controls:three frozen inputs, A/A calibration and
  four paired ABBA blocks,78 retained observations with every output verified.
  Median B/A ratios are0.945/0.947/0.960, but A/A scatter is larger than these
  differences, so no reliable timing speedup is claimed. Receipt
  `obj/parallel/stability-symbols-v1/measurement-v1/receipt.json`, SHA-256
  `b296e67effa8fd3cf1495f3c82bf0017b25c26732429eec5ab966f6bd8aedf59`.

Immutable CLI:
`obj/parallel/stability-symbols-v1/cli/182716780aaa97dd5c49808c26c5e9230d377d0af1f9f3b54ce52f17b2a4fa98/rustgm`.

Original core clears LIB-130's feature-index prepayment and reaches LIB-131:
`cfg_target_has_reliable_f16_f128`, the genuine lib.rs90 feature request. The
existing diagnostic is unlocated, `unsupported active source-core feature or
undefined library feature request`. A debugger breakpoint on that actual error
branch identifies the name and reads837493 charged work/12659641 storage bytes;
owner/reachability and the remaining inventory phases have not run. Therefore
the entire original inventory's budget remains required. No limit was increased.

The core observation takes18.7717s/375 opens, all722 source hashes and six
configuration hashes unchanged, with the same target/options/lint cap and finite
limits. Result SHA-256
`8a4cb9e16708692cc1062db339cc91c375d20bb1fefe781b25437efc0425eb8e`.
The diagnostic probe also preserves source and CLI hashes. Receipt
`obj/parallel/stability-symbols-v1/feature-probe-v2/receipt.json`, SHA-256
`5cf6ab7d45739d46d70354a06392907a97ae107cf9cd2e35ea9be828e4a4d6d7`.
Raw stacks, disassembly and decoded counter offsets remain alongside it. The first
debugger setup's unsupported printf precision remains a failed observation.

PA41/PA34/PA31 own the next feature:four genuinely gated backend reliability
cfg predicates, including math variants. Pinned rustc_feature unstable.rs and
builtin_attrs.rs bind the contract. Reading a predicate must not fabricate f16,
f128 or math capability. Implement real feature/macro allowances and target-profile
truth. Changing expand/config.rs requires an explicitly requalified configuration
binding; preserve the old manifest and source bundle, rather than claiming its
configuration-source hash stayed unchanged.

Full predecessor CI41748017598 on167968ee0 now passes every gate. Successor full
qualification remains separate from the focused evidence above. Failed naming and
format checks are preserved. Full Drop/Pin, imported nominal/value/body-policy
transport and original core/alloc/std artifacts remain required; no whole PA is
released by this optimization.
