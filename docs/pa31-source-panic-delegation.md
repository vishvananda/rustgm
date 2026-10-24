# Source panic delegation (LIB-013): reviewed initial scope

The preimplementation fixtures arrived as `af49643a`, cherry-picked as
`7815d710` on `pa31-panic-delegation`. Its base retains clean field checkpoint
`b937b6da`; the field branch is preserved separately and remains unqualified.
The original source contracts and predictions are unchanged.

Root's independent run `source-panic-independent-v1` passed all 39 contexts,
142 observations and 30 native programs. `review.json` beside the fixtures binds
the exact receipts and authenticates all sixteen retained source/contract files
against this worktree. It is independent Rust evidence, not candidate support.

The initial candidate scope is **actual local source-provider declarations in
admitted editions 2021 and 2024**. Both editions select modern handlers. The
2018 observations remain future edition evidence; this work does not add an
edition enum case or claim general 2018 behavior. That prerequisite belongs to
PA40 unless an earlier real crate manifest requires it. Foreign macro transport
and remapped defining-crate identity remain LIB-007. An original std source client
cannot claim provider roles merely because this local test profile admits a
`std_panic` builtin role on a genuine own-provider declaration. Its authorization
and full cross-crate protocol remain separate work. This slice cannot close all
of LIB-013 or supply a panic runtime.

## Pinned behavior

Rust revision `88d9e12ae178fab0fb5cc050a94da85685d449ea` implements this in
`compiler/rustc_builtin_macros/src/edition_panic.rs`. Its `expand` forwards the
original token stream into a new macro call. `ExtCtxt::std_path` in
`rustc_expand/src/base.rs:1337` supplies definition-site `$crate`, followed by
`panic` and the selected handler. `core_panic` and `std_panic` select panic
handlers; `unreachable` selects corresponding unreachable handlers.

`use_panic_2021` starts at the actual invocation span before the new builtin
expansion mark. At each outer expansion it skips to that expansion's actual
call site only when the definition carries the exact `edition_panic` allowance.
Otherwise it uses that definition's edition. A physical root span uses its real
source edition. An ordinary wrapper's edition therefore remains significant;
neither always using the physical caller nor always using the builtin declaration
edition is correct. Token lexical-edition provenance is a separate contract.

## Proposed ownership and APIs, before production edits

1. Add the three source-only builtin roles. Their explicit classification excludes
   them from bootstrap macro publication. `Body::Builtin` dispatch requires the
   existing actual source declaration and admitted own-provider identity; role
   spelling alone grants nothing. Local `CrateRoot(0)` is justified by that same
   package's genuine defining declaration. Missing handlers fail normal lookup;
   the authored placeholder body and opposite-edition handler are never fallbacks.

2. Place a `MacroContracts` value **inside the existing boxed `SourceCore` owner**,
   rather than adding a Package pointer. Initial client/nonprovider allowances
   remain unsupported. The cold owner retains validated interesting macro
   declarations and edition-walk suffix answers. A declaration record binds its
   actual unwrapped macro `NodeKey`, retained attribute owner and relevant exact
   attribute spans to its claimed role and narrow permission. It does not contain
   callable or ordinary library stability permission.

   A shared `Package::source_macro_contract(owner)` query normalizes the actual
   configured owner, validates active macro syntax and feature authorization,
   and publishes a complete record once. It uses interior mutation only inside
   this existing single-threaded cold owner, with checked reentrancy. Expansion
   definition publication and final source/stability collection consume that same
   record. Final collection must match the exact allowance attribute span; it
   does not accept `allow_internal_unstable` by name. Failed/partial records are
   never usable. Ordinary nonprovider/no-attribute paths allocate no new owner.

   Admit `allow_internal_unstable` as an implemented language capability request
   only for this narrow operation. Exact `edition_panic` syntax, enabled owner,
   macro placement and its feature gate are mandatory. Unknown allowance names
   retain an explicit unsupported frontier. Duplicate/malformed/unsafe-wrapper
   and unavailable-feature opponents cannot manufacture a permission. Repeated
   declarations are candidate opponents, not assumed Rust rejections without
   independent evidence. The existing const/default-body and other library
   stability rejection paths remain intact.

3. Retain each macro definition's actual defining `Edition` and the narrow
   producer permission alongside its existing cold source provenance. The
   current compiler computes the edition to compile a matcher, then discards it;
   a builtin body also discards the compiled matcher. No token, AST node, Body,
   Value, Type, Code or runtime frame gains fields.

   `Engine::panic_edition(call, original_span)` authenticates actual frame IDs,
   definition ownership and call-site contexts while applying the pinned walk.
   Cache only immutable non-root expansion-frame answers, sharing suffixes in
   `SourceCore::MacroContracts`; do not globally cache ROOT across physical
   editions. Owner work/storage charges precede parsing, walks and growth, retain
   attempted work on failure, and remain enforced on warmed queries. Actual cold
   definition/owner sizes and depth-by-width work must be measured when admitted.

4. Add a typed delegation builder under expansion. It creates path nodes for
   the authenticated defining crate, `panic`, and a typed handler enum; it builds
   the argument token forest from the existing classified invocation input.
   Only a diagnostic spelling image is constructed. No lexer, rendered-output
   parser, matcher invocation or eager argument expansion supplies semantics.

   Synthetic call/delimiter provenance remains call-site, while path identifiers
   carry the builtin definition-site context as in pinned `std_path`. Original
   argument origins, raw syntax contexts, lexical editions and joints remain
   unchanged. Interpolated arguments keep their actual registry payload identity.
   A narrow forwarded-only relocation operation can validate/remap existing
   registry handles without a fabricated `Matched` or fresh capture payload.
   Complete validation/reservation precedes publication of the new AST/capture
   owner and real invocation record. The selected handler then uses the normal
   macro worklist and argument parser. Invalid tokens are its actual rejection.

## Qualification contract

Reuse authenticated independent observations; do not invoke the oracle again for
unchanged inputs. Select candidate cases only for the admitted local edition
profile, retaining 2018 and foreign rows as positive pending requirements. Add
focused producer/consumer controls for exact claimed owner and role, absent or
malformed permissions, wrong frame/call site, ordinary versus skipping wrappers,
warm identity checks, shared depth/width work, and forwarded capture identity,
edition, spacing and diagnostics. Existing source-role forgery and shadowing
controls remain required. Permission syntax opponents need independent status
before being labeled Rust-negative.

After the implementation qualifies, retry unchanged core with the exact original
source/configuration hashes and record the next real frontier. Recognizing its
declaration alone is not a successful whole-core build. No new worker build,
oracle, test or cleanup has run for this proposal.
