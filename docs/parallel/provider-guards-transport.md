# Provider-body guards and single-variant matches

This is a Lane C checkpoint in `rustgm-async-work` on
`implement/trait-methods-20261107`, based on `c6467111a` (the joined
inherent-methods checkpoint). It is a transport slice, not a library
milestone, and makes no claim about an original-core artifact or a whole
assignment. It covers two things:

- match guards in version-13 provider bodies;
- `match` and irrefutable `let` on a single-variant enum in provider bodies.

## Single-variant matches

Local lowering emits no constructor test for a variant whose enum has only
that variant, because the pattern is irrefutable. Version 13 exported such a
body, but every fresh consumer rejected it with "source downcast field use is
not dominated by its variant test", because the consumer's variant-fact proof
asked for a test that was never emitted.

There is now one rule. `Members::needs_test(variant)` is true unless the
variant's enum has no other variant. Local lowering (`pattern_tests`,
`guards`, `patterns` and `retries` under `body/build/`) and the consumer's
`variant_facts::proved` both call it, so the consumer accepts an untested
downcast exactly when local lowering would have emitted none. It reads the
consumer's own installed enum, not a wire claim. For every valid variant the
local condition is unchanged.

## Guards (version 13)

The wire carries the local IR's guard forms. No protection or decision proof
is transported.

| Form | Tag | Operands |
| --- | --- | --- |
| guard view local | local kind 3 | guard ordinal (`u32`), no declaration |
| `GuardStart` | action node 2 | `[Flag(guard)]` |
| `GuardUse` | action node 3 | `[Flag(guard)]` |
| `BindView` | action node 4 | `[Local(view), Place(place)]` |
| `Select` | action node 6 | `[Projection, Place(anchor)]` |
| `FalseEdge` | terminator node 1 | `[Block(real), Block(imaginary)]` |

Guard columns follow the calls exactly when a statement is a `GuardStart`:
guard rows `(tested range, view range)`, then the tested places. Presence is
derived from rows that are already decoded, never from trailing bytes, so
originals without guards keep their exact bytes. Out of line, the
`Original` record holds them as `guards: Option<Box<Guards>>`.

- **Export.** `lower/transported_source/cfg_rows.rs` maps the guard actions,
  the imaginary edge and guard view locals under `body_nodes()` only. Every
  older stage keeps "source guard facts require a wider transport profile". A
  guard view drops its declaration, because its arm's binding local already
  owns that `DefId`; exporting it twice failed the declaration-owner check.
- **Codec.** `ir_nodes.rs` admits action forms 0 to 11 and terminator forms 0
  to 5. Supported codes with wrong operands are rejected as malformed, and
  other codes by name. `cfg_columns.rs` writes local kind 3 only through the
  body encoder. The branch encoder rejects `FalseEdge` with "source imaginary
  CFG edges require the version-thirteen encoder". The legacy generic codec
  (`originals/bodies.rs`) rejects both by name.
- **Descriptor.** `branch_profile.rs` rejects any guard row, guard action,
  guard local or imaginary edge under an older profile with "source guard
  facts require the version-thirteen body IR profile". It charges one step per
  statement, local and block.
- **Validation.** `body_ports/cfg_storage.rs` charges rows, places, locals and
  statements, then checks the following:
  - every guard ordinal lies in the table;
  - every guard is started;
  - the tested ranges slice the tested places;
  - the view ranges lie within the locals and do not overlap;
  - each `BindView` binds a guard view local;
  - each guard view lies in its own guard's range.

  The snapshot walk (`ssa_ports.rs`) refuses an imaginary edge as it refuses
  a branch.
- **Consumer.** `rebuilt_operations.rs` rebuilds `Body.guards` and
  `Body.guard_places`, maps the actions (with `Select` through the regional
  projection) and the imaginary edge, and requires the rebuilt body to equal
  the decoded one. The local guard ownership proof
  (`borrow/ownership/guards.rs`) then runs unchanged: tested places are
  protected shallowly and views deeply. `variant_facts.rs` follows only real
  edges, so an imaginary edge contributes only its `real` successor. Each
  `BindView` and `Select` requires a live constructor-test fact for its place,
  and a guard view is killed when it is rebound.

The spec.md version-13 guard paragraph records the wire rule, and the enum
paragraph no longer lists guards as rejected.

## Forgeries

`forged_guard_rows_reject_in_fresh_consumers` uses a generated minimal
provider (the foundation plus `large` and `guard`). In that provider each
guard start, guard use, view binding and imaginary edge is unique in the
packet.

| Mutation | Fresh consumer |
| --- | --- |
| guard view relabelled to kind 1 | "source guard view binds an ordinary local" |
| `GuardStart` ordinal past the table | "source guard ordinal is outside its guard table" |
| `GuardUse` ordinal past the table | "source guard ordinal is outside its guard table" |
| `BindView` view set to the parameter | "source guard view binds an ordinary local" |
| real edge retargeted past the constructor test | "source downcast field use is not dominated by its variant test" |
| relabelled version 12 | "unknown source declaration kind" |
| every strict prefix | fails to decode |
| real and imaginary successors swapped | accepted (sound) |
| tested range emptied | accepted (sound) |

**Rework, recorded in the outcomes file.** The swap was declared as a
rejection. It was accepted because the swapped body is a different but sound
program: execution always takes the fallback arm, and the guarded arm is
reachable only through the edge that never executes. This follows the
callable-role precedent of the accepted two-phase relabel. The swap stays as
an accepted control, and the skip-test control was added before it was run.

The emptied tested range is sound for a different reason. The consumer
re-proves every downcast by its own live constructor-test fact, which a write
to the scrutinee kills. Dropping the shallow protection therefore admits only
a body that rustc would reject for mutating its scrutinee in a guard.

## Requested control on the `abcfea354` behaviour

The requested control was a forged older-profile associated-method body with
a forged result lifetime, and it was not constructed: no older-profile packet
of that shape can be exported. Under version 12, the provider export rejects a
trait or inherent method with late regions ("source original region formals
require their complete wider body profile"). The variants tried were:

- a static associated `first`;
- `get<'a, 'b>(&'a self, ...)`.

Both stop at export with "source adjusted/unsizing coercion requires a complete
wider input profile". That happens in both the `abcfea354` baseline binary and
the candidate, under both version 12 and version 13.

So the hole cannot be reached through an older-profile associated method, and
the version-13 free-original control from the previous checkpoint
(`late_region_originals_reprove_their_declared_results`) still rejects. The
probes are in `obj/parallel/provider-guards/older-forge/`.

## Amended controls

- `pa29/tests/source-body-ir-outcomes.json`: terminator code 1 is now a
  supported form, so its relabel gives "body IR terminator node has malformed
  operands". A new code-6 control keeps "unknown body IR terminator form".
  `node_transport` checks both.
- `pa29/tests/enum-match-outcomes.json` and `enum-methods-outcomes.json`: the
  guarded control now exports under version 13, and a fresh consumer of
  `guard` reaches LowIR. Both entries are marked `amended`, and
  `match_transport` checks the new outcome.
- The generated source-nominals `either` control stays fenced under
  versions 12 and 10. Under version 13 it exports and its client exits 42.
  The first client's own `match` on a non-`Copy` `bool` was rejected, so the
  client became an `if`. The oracle's copy of the provider needed a
  `#[cfg(rustc_oracle)]` `bool` `Clone`/`Copy` shim, because rustc 1.98.0
  stopped with an internal error without it. The candidate input is unchanged.

## Edits outside body transport, for owner review

- `sema/members.rs`: the new `Members::needs_test`.
- `body/build/{pattern_tests,guards,patterns,retries}.rs`: each replaces its
  inline `variants(owner).len() > 1` with `needs_test`, with the same
  condition for every valid variant.

## Evidence

The fixtures are in `pa29/tests/spec/provider-guards/`, and their outcomes,
declared before the oracle and the candidate ran, are in
`pa29/tests/provider-guard-outcomes.json`. The integration tests are in
`crate_linking::guard_transport`.

- **Native.** The eight accepted clients exit 42 natively, as rustc's do:
  guarded, fallback, retry, predicate, method, single, single-guard and
  own-single. So does the generated `either` client.
- **Opponents.** All three are rejected, as rustc rejects them:
  - moved-scrutinee: E0382 / "uninitialized or moved";
  - wrong-predicate: E0271 / an unsatisfied obligation;
  - wrong-single: E0308 / "incompatible types".
- **Scaling.** Packets are 38,236, 68,674 and 190,426 bytes for 6, 24 and 96
  guarded functions. That is 1,691 bytes per function, exactly linear. The
  client LowIR is 2,556 bytes at every width.
- **Byte identity.** The baseline `c6467111a` (sha `294f7b99`) and the
  candidate (sha `3bac293d`) were compared over the older v10, v12 and v13
  fixtures. Across 482 metadata, LowIR, status and diagnostic files, none
  differ.

| Check | Result | Receipt (`obj/qualification/fast/`) |
| --- | --- | --- |
| guard, node and match integration (6) | 5 pass; older-gates test fixed in the next row | `1794069650953855034` |
| older-gates test after the fix | pass | `1794069865971828352` |
| native `--ignored` (guard, match, constructor, inherent) | 4/4 | `1794069909944314569` |
| neighbouring v13 suites (inherent, constructor, match, node, branch, fn roles) | 20/20 | `1794070026285927938` |
| private selection (7) | 6 pass; hot-record fails only at `Imported` (216, asserted 160) | `1794070322423528132` |
| PA1-PA12 | pass | `1794070354103689145` |
| PA13-PA19, PA21-PA24 | pass | `1794070418051405706` |
| PA20 | 1,451/1,451 | `1794070495816852665` |
| `make test-upstream` | 127 fixtures, 126 references | `obj/parallel/provider-guards/test-upstream.log` |

The rustc 1.98.0 receipt is `obj/parallel/provider-guards/rustc/receipt.txt`,
and the identity receipt is `obj/parallel/provider-guards/identity/receipt.txt`.

## Pending

- The hot-record invariant still fails at `Imported`, because of Lane B's
  macro rows.
- The `fn_allowances` transport case
  `genuine_original_const_bodies_roundtrip_and_execute_without_client_feature_requests`
  belongs to Lane B and was not selected.

## Remaining gaps toward `core::option::Option::map`

- const callable bounds (`[const] FnOnce + [const] Destruct`);
- lang and diagnostic items, stability attributes, derives and `must_use`;
- closures in provider bodies;
- callable bounds written by the consumer;
- trait impls with methods on provider nominals;
- `&'static str` and `!` in transported signatures and bodies.
