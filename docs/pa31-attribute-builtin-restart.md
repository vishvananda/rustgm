# LIB-019 attribute builtin auth-pause restart

User requested an immediate authentication pause. No C heavy command, probe,
oracle, network action or public push is running or queued. Resume only after
parent/user authentication resumes the work.

Branch: `pa31-attribute-builtin-declarations`; HEAD `5a1bd038d482296fa73970d453a8f611b395251b`.
The qualified byte dependency is `a3d7dc49`. Frozen data-only successors are
`6e5b736d` (37 declaration/application hypotheses) and `5a1bd038` (8 additive tool
hypotheses). All 45 are unobserved. Their expectations and source bytes must stay
unchanged; preserve future disagreements rather than rewriting them.

The production/test draft remains dirty and UNCOMPILED/UNQUALIFIED. Exact file
hashes and the tracked-source patch are retained under
`obj/parallel/pa31/attribute-builtins-auth-pause-v1/`. Untracked new files remain
in the worktree and are included in that snapshot's hashes. No source reset,
rebase, cleanup or dependency integration has occurred during this pause.

## Exact current typed interfaces

- `BuiltinKind::{Bang,Attribute}`; two appended `Builtin` roles
  `AutodiffForward`/`AutodiffReverse`; `Builtin::{kind,executable}`. Both attributes
  are declaration-only with unavailable execution. Existing role indices stay
  fixed. Public AST `MacroContext` is unchanged.
- Internal `CallGrammar::{Bang(MacroContext),Attribute}` replaces the existing
  `Call.grammar` field. Bang consumers call checked `bang()` extraction. Attribute
  calls retain a checked configured-effect index in the existing `capture` slot;
  it is never interpreted as a bang capture.
- `Engine::create` factors the exact existing production loader/initialization/
  initial-root worklist once. Both production `analyze_with_context` and private
  controls use it; there is no alternate reconstructed engine/Call/CFG.
- `Engine::attribute_call`/`macro_attribute` inspect actual active configured
  owners, preserve real Forest paths/spans, queue the first macro attribute via
  existing ready/import-wakeup machinery, and block owner children. The existing
  node-scope index binds the actual attributed wrapper to its caller scope.
  `decorated_calls::validate(&Package,&Resolution,&Call)` authenticates owner,
  effect, active state, module/file and exact retained caller-scope binding.
- The retained Forest path is projected directly into existing path Buffer/Saved
  with actual ItemContext and canonical Names. The parser already lowers opaque
  paths through `Splice::append_attribute_path`; no fake AST node/reparse is used.
- `require_invocable(definition,kind,file,offset)` checks actual header/body kind
  and unavailable execution before capture/matching/mark/Invocation work.
  Public builtin expansion and allowance FrameView defensively deny these roles,
  including a forged executable body with the actual attribute header.
- `support::decorations::Consumer` names existing compiler attribute consumer
  families; representation, linkage, target-feature and safety-lint validators
  use it while retaining actual validation. No prefix/name-list grants semantics.
  Unknown active ordinary attributes become real macro lookup failures. The exact
  five compiler-predefined tool identities retain tool metadata only; source
  `register_tool` explicitly remains unsupported.
- Source budget charges precede walks, canonicalization and allocation. Failed
  warm checks retain work. No Body/Code/Frame/Machine/Package fields or new proof
  engine were added. Appended roles increase the existing builtin-slot Vec's
  payload; this is not yet measured. Call/Engine/Body/Definition and ordinary
  controls must be measured, not inferred from enum arithmetic.

## Test and validation completeness

Seven private tests are authored in `attribute_declarations.rs` and
`decorated_owners.rs`: actual compile-once declaration/import width8/32/128,
selected-to-denied zero execution delta (including generated/captured inputs),
macro kind opposition, actual owner/effect/scope and warm failed-budget controls,
unknown attribute blocking nested work, authentic header/body frame forgery, and
ordinary no-attribute actual engine storage/layouts. They have NEVER run. The
source formatter parsed/formatted the draft successfully after correcting two
private raw-string syntax errors; this is not audit/Clippy/test qualification.
Manual current-API/function-length review and affected frontend/native controls
are still incomplete. Initial `invocation_kind.rs` was renamed
`application_kind.rs` after read-only filename-family inventory reached7.

Oracle runner and plan are NOT YET WRITTEN/FROZEN. Intended next command, only
after resume and real admission, is:

```sh
python3 scripts/resources.py run --max-output-gib 8 -- \
  python3 scripts/storage.py run \
    --prune-under obj/parallel/pa31/attribute-builtins-independent-v1 -- \
  python3 obj/parallel/pa31/attribute-builtins-independent-v1/run.py
```

Freeze that runner/plan and all original source/tool/native consumer hashes first.
Expected corpus is 45 sources, 22 predicted native positives and 23 negatives in
both editions (268 planned observations:180 metadata/object and44 linked native
programs). Retain actual versus planned counts, tool failures and disagreements.
Current entire-obj read-only UsageScan observed7,644,422,144 allocated bytes;
recheck actual usage and headroom before C8 admission. A later candidate gate is
proposed C12, not admitted/frozen. Reserve150GiB/CPU8/memory12GiB/tasks256 and
Cargo/test2 remain unchanged. No unchanged admission retry.

After oracle review, finish focused frontend/native controls and freeze fresh
format/audit/strict -> one debug library/frontend/CLI build -> full private once
-> affected source/native -> same-CLI unchanged original core. Keep the accepted
byte CLI/result pinned. Original core last genuinely advances to attribute
`autodiff_forward` line1553/byte57236; no core artifact exists. A new classifier
may honestly expose an earlier unsupported compiler attribute; never bypass it.

## Integrator seam and separate prerequisite

Root owns newer `Body::Scoped`, real executed owner/scope allowance authentication,
`require_invocable` and `install_builtin(mode)`. Merge the tiny attribute guard/
CallGrammar additions explicitly, preserving those changes; do not copy older
whole files. No C heavy action remains from this checkpoint.

B's early Copy readiness passes the old ed3 Copy failure and now reaches the
selector caller-owner boundary. Preserved LIB005 branch `pa28-anonymous-select`
HEAD `379006ad3fb2f950a1ce2df8f5db9020abe06775`, production `ee0a1fa8`, is clean but
UNQUALIFIED. Its anonymous caller explicitly rejects the selected-reference
profile, so it does not close original reference-anonymous-selector even with a
closed usize whole result. No CTFE gate was weakened. B and root have this exact
status; further Row/opening/liveness work needs reviewed scope.
