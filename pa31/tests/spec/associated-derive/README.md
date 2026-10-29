# Typed associated equalities in source derives

LIB-078 connects PA18 associated predicates, PA26 derives and PA31 source-owned
bindings. `contract.json` froze inputs/options/predictions before observation;
this is a planned cluster sidecar, not assignment activation.

`upstream-decode.rs` is the exact unchanged DecodeUtf16 declaration/attributes
from Rust88d9 `library/core/src/char/decode.rs`, bytes357–533. Its `Item = u16`
starts at482. The failing syntax is that equality, not the Option field.
`original-expansion.rs` composes it with genuine source macro declarations;
`original-rust.rs` independently observes the original under pinned Rust.
Primary and fragment hashes are in the contract. Preserve Rust's attribution and
MIT/Apache-2.0 licensing; see root NOTICE and third_party/sources.json.

The source type copier now handles the existing typed Constraint node by copying
its actual target, optional value and bounds list through the existing paid
selected-subtree walker. It preserves NameIds/spans and where-clause structure;
there is no rendering/reparse, dropped equality, whole-source append or new
semantic permission. Unsupported descendants retain their named wider-profile
boundary, including associated bounds requiring projected obligations.

The direct transport invariant checks nested generic values, original name/span
identity, exact subtree scope, invalid node IDs, initial and mid-copy failed
payments, and N8/32/128 linear work/storage. These copier-only types are syntactic
inputs. Semantic admission is checked separately with a real authored Family
trait and `Family<Item=u16>` source Clone consumer. Its non-Copy payload has an
actual Clone body. Wrong equality, missing associated member and missing Clone
are semantic opponents. That authored Family never replaces original Iterator,
Option or Debug implementations.

All11 independent Rust operations matched the frozen contract, including original
and nested expansions/metadata, negative metadata and the exact source native7.
Four focused candidate controls pass: actual original Clone+Debug headers,
transport/payment/scaling, the source Clone/opponents/LowIR exit and the existing
bounded generic source control. The actual LowIR/object/C-native program prints
`7`, matching independent Rust. Upstream verification and the exact original byte
interval also pass. The initial test-only LexError/ownership static failure is
retained; no compiler API or whole-unit Clone was added to work around it.

Measured copies for N8/32/128 have50/194/770 nodes, work74/290/1154 and charged
bytes7968/31008/123168. Syntax Node36 is unchanged; no production record fields,
profiles or wire contracts grow. Receipts and source bindings are retained under
`obj/parallel/associated-derive-v1/` in the worker checkout.

Whole original DecodeUtf16/Iterator/Option/Debug runtime, full original library
artifacts, wider quantified/associated-bound descendants, metadata transport and
parent PA release gates remain Required. Root owns merged production/CLI and the
next unchanged original-core observation; this checkpoint makes no artifact or
whole-library claim.
