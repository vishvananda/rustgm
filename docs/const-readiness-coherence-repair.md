# Const readiness and coherence repair

Public checkpoint9da151884 cleared reservation implementations in the original
core probe, but full CI41013188274 exposed seven regressions. Source84e73fcfff
repairs all seven in focused development qualification; a fresh complete CI run
is still required before claiming current whole-compiler qualification.

Ordinary const type readiness must not construct an unused defining CFG or drain
pending pattern work. Complete lowering still checks unused ordinary and generic
const bodies, while genuine compile-time-only functions retain eager eligibility
checks. Promotion facts are prepared through the existing engine before retaining
the actual concrete original. Existing source inputs and full compile outcomes
are preserved; unsupported reservation metadata keeps its explicit admission
boundary rather than pretending to export an incomplete crate.

Coherence now instantiates and equates implementation heads before expanding
conditional assumptions. Incompatible heads skip that unrelated work. Compatible
heads retain separate type/const substitutions and lifetime mappings, then run
the full condition checks. Reference/iteration queries continue to materialize
all conditions. No resource limit, existing negative outcome or proof obligation
was relaxed. The specification records this order and its scaling obligations.

## Executed qualification

- B source8e1faab60: all249 focused private controls pass in124.61 seconds,
  including the original borrowed-constant regression, wide computed requests
  at8/32/128 and new type/metadata/full-lowering cutpoints. Strict Clippy,
  formatting, architecture and assignment metadata pass.
- The same source passes the four unchanged initializer/projection/atomic
  integration regressions in117.29 seconds, with production Clippy and actual
  CLI/test builds.
- Exact main84e73fcfff: all18 selected coherence/cache/payment/lifetime controls
  and three complete trait/reference/iteration integration matrices pass in
  254.73 seconds, including the formerly failing repeated-condition chain and
  new8/32/128 head-versus-condition scaling. Strict production and private
  Clippy, formatting and architecture pass. All3242 source fingerprints match
  the declared main snapshot.

The last check reuses B's existing warm targets. Two attempts in C's other cache
passed static/ordinary Clippy checks but timed out compiling the private harness;
no runtime was executed there. Their failed receipts remain recorded. None of
these checks used a release build, raised the five-minute/16GiB limits or dropped
a declared case.

Receipts are retained in the B worktree under obj/qualification/fast:

| Check | Receipt directory | SHA-256 of receipt.json |
| --- | --- | --- |
| Private249 |1793343608644382610|1942fa3bb66475828faea0754391cb3d2fd78602e59918c6de7695513747ca26|
| Integration4 |1793343758682065176|a7d6f36079ea38fa112ca6d9df1a77a38a5a7bafbec42f81d6407df01ff57b6e|
| Main21 |1793344746838990967|845c1c8c1050c09a083287c19befb631f885d99cdee66465b8d77bd977e5ed21|

This repairs compiler regressions. It does not complete a whole assignment or
produce original core, alloc or std. The latest original-core observation is
still the qualified9da source reaching rustc_has_incoherent_inherent_impls on
core::error::Error; current conversion transport and the new source permission
contract remain separate work.

## Follow-up expectation reconciliation

Full CI41019939070 clears the original seven failures and exposes three stale
tests. Source bb2013ef2 changes no production code. Both selector assertions now
include the existing one-step function-owner eligibility probe; per-call costs,
8/32/128 scaling, sparse descriptor storage and zero-work early exits remain
checked. The provided-default test now requires the two previously declared
Rust-positive const-default programs to compile, and the existing native test
executes both alongside its original runtime/cleanup opponents.

The corrected exact selection passes six private and eight integration controls
in34.24 seconds (B receipt1793347065603287970), including strict Clippy and all
static checks. The existing native case executes all eight programs in30.35
seconds including guards (B receipt1793347157520753214). Fresh pinned Rust
object/link/native observations repeat both required29 answers in0.258 seconds
(main obj/parallel/ci-default-selector-v1/rust-v1/receipt.json). Original fixtures,
Rust expected outcomes, implementation checks and resource quotas are unchanged.
The first local selection attempt compiled successfully but selected a submodule
instead of exact case names, so it ran zero tests; that failed receipt is retained.
A fresh full CI successor is required; the predecessor failure is not green.
