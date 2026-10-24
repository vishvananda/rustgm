# Original core `cfg_select` contracts

The qualified scoped-declaration component reaches core's `cfg_select` at
`library/core/src/macros/mod.rs:235`, after passing `assert_matches` at line 170.
The fresh probe confirms LIB-015; it does not complete core expansion.
Production builtin support and combined release qualification remain pending.

The [source corpus](../pa31/tests/spec/cfg-select/README.md) records 61
`cfg_select` sources across seven immutable predeclared contracts. One additional
unchanged source-provider `assert_matches` fixture is independently checked in
the initial batch. Seven runs execute **206 observations**: 124 metadata/object
checks and 41 native programs, each linked from its actual generated object.
Of those native programs, 40 exercise `cfg_select` and one the scoped declaration
provider. See the [observed inventory](../pa31/tests/spec/cfg-select/observations.json).

The original run has five reviewed source mismatches, retained in
[review.json](../pa31/tests/spec/cfg-select/review.json). Two negatives reject as
predicted but with different diagnostic reasons: an empty selected branch at a
brace-delimited statement tail produces unit (and fails the requested `u32`
return), and an invalid later
predicate reports E0805. Three predicted positives stop earlier because the
minimal `no_core` provider lacks the Copy language item. Additive fixtures supply
genuine Clone/Copy/receiver declarations; all three then compile and execute.
The original sources and hypotheses remain unchanged. A separate unit-returning
statement-tail control also compiles and executes. The historical fixture names
do not determine their actual fragment category.

Independent observations establish these implementation requirements:

- Select the first matching predicate, otherwise the first wildcard; report an
  error if no branch matches. Later and repeated wildcards are accepted with
  lints capped, but their branch syntax remains checked.
- Parse every branch in the invocation's fragment category. Unselected malformed
  expression, item and type syntax rejects; unselected unresolved names and
  macros are neither resolved nor expanded. This also holds after a wildcard.
- Support expression, statement, type, pattern, item, inherent impl, trait,
  trait impl and foreign-item positions, plus braced/unbraced branch framing.
- Preserve actual captured token provenance and opaque metadata/expression
  inputs. Both selected and unselected opaque-expression controls pass.
- Authenticate the builtin role on both `pub macro` and `macro_rules!`
  declarations. An ordinary macro with the same spelling keeps its authored
  behavior. Renamed imports and qualified paths select the actual declaration.
- Consume the explicit cfg inputs. The configured-on/off controls return
  different independently observed answers from their declared configurations.
- Preserve statement versus expression context. Forced initializer and
  parenthesized expressions reject an empty selected or unselected branch.
  Brace-delimited statement tails and explicit statements permit empty or
  item-only fragments. A dedicated native control returns the selected tail
  value `17` through the C ABI.
- Preserve an opaque captured expression's category, including when its outer
  macro invocation has a semicolon. Raw token forwarding can enter statement
  grammar, but an outer parenthesized invocation at block tail is an expression.
  The original contrary raw-token-tail prediction remains a recorded mismatch;
  a distinct brace-delimited forwarding control compiles and executes.

The exact original builtin declaration has pinned source/slice hashes and
MIT/Apache-2.0 attribution. It compiles and executes inside a complete actual
provider. Width 8/32/128 and selected depth 8/16/32 are frozen performance inputs;
their independent native answers pass, but RustGM scaling measurements have not
yet been made. Do not infer a complexity bound from the native answers.

Evidence remains under `rustgm-pa29-work/obj/parallel/cfg-select-oracle-v1`
through `v7`. The [context review](../pa31/tests/spec/cfg-select/context-review.json)
binds the additional observations and retained forwarding mismatch. Failed
runs were diagnosed and explicitly pruned;
successful additive runs clean their generated objects and executables.
Every source, diagnostic, receipt and checksum remains. PA31 stays planned;
no reference text or assignment activation is part of this contract handoff.
