# Function-pointer source contracts and address shims

PA31 owns actual `fn_ptr_trait`/`fn_ptr_addr` declarations; PA20 owns trait
evidence, PA26 the selected method hook, PA27 pointer/safety distinctions,
PA25 const effects and PA29 imported identities and bodies. Whole assignments
remain planned. This packet qualifies a local source and ordinary runtime
profile, not full language or library completion.

`contract.json` freezes 38 complete authored source inputs, predicted Rust
outcomes and hashes. `cases.tsv` is their executable manifest. Pinned Rust
1.98.0 (88d9e12ae178fab0fb5cc050a94da85685d449ea), edition 2024,
independently confirms every outcome. These are authored protocol tests, not
verbatim upstream imports. The foundation retains the inherited sizing provider
and existing Copy/Clone/Freeze/Destruct/drop-glue conventions; repository NOTICE
and third-party attribution apply.

Primary sources in the pinned read-only Rust checkout are:

- `compiler/rustc_hir/src/lang_items.rs`: trait target/zero generic arity and
  required trait-method target with unrestricted method generic arity.
- `compiler/rustc_trait_selection/src/traits/select/candidate_assembly.rs`:
  builtin evidence applies to actual function-pointer types.
- `compiler/rustc_ty_utils/src/instance.rs`: exact method/trait identity selects
  a function-pointer address shim.
- `compiler/rustc_mir_transform/src/shim.rs`: one pointer read and conversion
  to `*const ()`, without calling the pointed-to function.

Cases distinguish source owner, feature permission, duplicate claims, method
body/target, actual bounds and explicit implementations. Safe, unsafe, C ABI,
higher-ranked and non-Copy-signature function pointers have real evidence;
function items, closures, raw pointers and references do not acquire it directly.
Generic callers need their actual bound. Reification and method calls retain
their real selected instance. Unused Rust-valid generic, borrowed-result,
orphan and other-result declarations must not acquire invented signature
restrictions merely to fit the standard library's method.

The candidate runs all 38 cases through source analysis, original body proofs
and demanded lowering. Its standard demanded address shim requires the actual
FnPtr-owned method, closed function-pointer Self and safe ordinary Rust
`fn(Self) -> *const ()` signature. The native companion covers direct, bounded,
unsafe-pointer, reified, once-evaluated receiver, higher-ranked and scoped
lifetime calls. The C observer sequences calls before reading counters. Both
pinned Rust and emitted LowIR through the locked CPPGM object backend print
`1\n1\n1\n1\n1\n1\n1\n0\n1\n`: seven address equalities, zero callback calls,
one receiver-producing call.

Broader demanded method signatures, const effects, imported role/body transport
and all real library graphs remain required. Declaration acceptance alone is
not execution qualification for those profiles. Unsupported imports retain
their existing explicit fence. Private forged-body/descriptor/lifetime tests
compile under strict all-target Clippy and execute successfully in full
CI41929736419 onf94868db5.

[The checkpoint](../../../../docs/pa31-function-pointer-checkpoint.md) records
the qualification receipts, performance observations, retained failed attempts
and the unchanged original-core successor boundary.
