# Actual operand borrows for language calls

This checkpoint fixes borrowed Deref/DerefMut, Index/IndexMut and assignment
calls that previously borrowed the opened callee receiver type instead of the
actual operand type. Nested reference lifetimes can differ between those types.
The existing comparison borrow producer now accepts the selected mutability and
serves these calls as well. Comparisons still request shared borrows.

The physical Borrow retains the actual referent exactly. The selected callee
receiver remains a separate expected type, with its existing shape constraint,
directed coercion and region/loan obligations. The existing body validator still
rejects an unequal Borrow referent; its ordinary checked conversion runs once
after the borrow. No nominal equality, lifetime equivalence, const permission or
move-through-reference exception was added.

Index publication rechecks the selected call against its existing indexed
Pending and the same Solved result used for publication. This is a constant-time
lookup, without a coercion-vector scan or new authority map. Const-index defining
validation checks the actual Index operand, its recorded built-in reference
dereferences and borrow mutability independently from the exact expected
receiver, signature and reference-result lifetime. Existing const execution and
body proof remain required. The added traversal uses the existing Work budget.
No dense record or metadata representation changes.

The PA31 Deref packet adds `nested-borrow.rs` and `nested-observer.c`. It uses
only already-supported Pin/Deref/DerefMut roles. A generic callback receives a
borrow through `Pin<&mut &mut S>` and writes 23 to the original scalar. Source
SHA256 is `0889de320b09aa07250b5d186b961eed43a13a813ed0714438bef149649b98a1`;
observer SHA256 is
`eb4fd6aabd9683553211c89161be74f1794d4a6b542de31e3692d5377ed03ba2`.
Both prediction and source were frozen before candidate observation.

Pinned Rust 1.98.0 independently accepts the source and its native observer
prints exactly `23\n`. The immutable baseline CLI with SHA256
`9ebf8ae50fa4c54aa9fd2ed59bd5f2e0fd6728621b42271c1b723f8a193e5415`
rejects the same source at byte 3920 with “method borrow differs from selected
receiver place”. The first reducer lacked a genuine Unpin bound and tried to
return a borrow of a local Pin: its E0596/E0515 observation and source are
preserved separately, without a parity claim. A first baseline invocation also
omitted the library flag; its setup failure is preserved.

Local evidence under `obj/parallel/async-iterator-v1/borrow-regression/`:

- `oracle.json`: Rust/native/baseline comparison, 0.694s, receipt SHA256
  `1ac4a27167810dd8269716f79a1a19509fbe4a76cc86ae9163880422034be76a`.
- `native/receipt.json`: the exact qualified frontend/traits artifacts execute
  the new nested borrow, seven index observers, nine assignment observers and
  the existing Deref effect/evaluation-order observer. All 18 observers pass in
  5.296s. Receipt SHA256
  `57ad3c50574cb29b22bda6386e315fd874475865377452c1ecf87c6850da04d8`.
- `initial-invalid/` and `oracle-no-library-setup.json` retain the failed
  independent/setup attempts. No failed outcome was relabeled as candidate
  semantic parity.

Focused gate `obj/qualification/fast/1793937875081586191` passes metadata,
formatting, architecture, strict Clippy and seven exact tests in 98.37s. These
include the 32 Deref cases, source Index/IndexMut causes including const-valid,
all 138 assignment causes, legacy operators and comparison controls. Its receipt
SHA256 is `efeea4c67b235e54aae28db5029ca9dc27c5d6d93235be172aea3538187ec0a9`.
The subsequent native run verifies every recorded dev-source and binary hash
before execution. Upstream validation passes 32 pinned files and 126 Reference
files. No timing speedup or broad regression-absence claim is made.

That gate also checks the complete original AsyncIterator forwarders/helpers
under the worker's surrounding uncommitted role/receiver prerequisites. Those
files and their 35-case source/native packet are outside this separable commit.
Its passing whole-source observation does not qualify those prerequisites as
independently integrated. Full stream manual field-autoderef, four stream native
observers and the Rust-positive alias128 selection failure remain required and
unchanged. Direct nesting 128 retains the existing syntax-depth limit and its
failed observation. Generated async/gen state, suspension, capture cleanup,
cancellation and transport remain required; this borrow repair grants none of
their execution authority.

Five new private const-index forgery opponents cover the expected receiver,
borrow absence/mutability/referent and source operand. Their compilation and
execution remain pending full CI; no cold private harness was linked locally.
The previously recorded TMPDIR/controller-discovery harness setup failure stays
pending Root's existing integration configuration, without a harness change.
