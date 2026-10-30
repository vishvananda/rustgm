# Source const-item-call advisory

The pinned `rustc_should_not_be_called_on_const_items` parser accepts a word on
actual inherent or trait-impl methods (including associated functions). It
requires `rustc_attrs` or genuine macro-definition allowance and rejects active
duplicates, free functions, trait declarations/defaults, foreign declarations and
closures. Macro allowance follows the actual expansion context; generated meta
attributes and passthrough item origins have separate source controls.

This is not a const-call prohibition: const eligibility, unsafe permission,
shared-reference mutation and real method/body contracts retain their ordinary
rules. Rust's `const_item_interior_mutations` advisory checks direct method/UFCS
receivers that name const items, excluding deref-adjusted receivers. Broader
warning generation and allow/deny/expect parity remain required, deferred work;
the four lint-policy reference cases are not candidate language-rejection claims.
The existing lint controls for implemented diagnostics are unchanged.

This source phase reuses the paid transient owner pass, authenticating the actual
impl source, member/Self scope, function identity and source image before
publication. It adds no persistent absence API or common Node/Header/Body field.
No current language/ABI/backend consumer uses the marker as permission; future
imported advisory production must preserve genuine intent rather than infer
absence from omitted wire data. No new metadata privilege or schema is granted.

`contract.json` freezes all source hashes and independent metadata/object outcomes
before execution. It records exact pinned Cell/LazyCell source intervals and
whole-file hashes. Original algorithms, unsafe blocks, APIs and full artifacts
remain required; these fixtures do not substitute for original Cell execution.

Pinned Rust metadata/object observations matched all 76 successor outcomes. The
first run matched 70/72: the meta-capture allowing macro was wrongly predicted
to reject. Original source and prediction history are retained. Genuine macro
meta emission accepts; a passthrough item and a macro without the allowance
reject. Candidate qualification and measured transient costs remain pending.
