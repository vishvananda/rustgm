# Cold advisories on no-body declarations

This follow-up closes the two independently accepted no-body inputs from the
cold closure checkpoint: a required trait method and a foreign function. Cold
uses ordinary target/lint policy. Required trait methods receive an
unused_attributes warning (Deny rejects); foreign functions are permitted, and
duplicate words retain the existing policy.

The consumer captures actual owner/effect rows in the existing local header
ledger. SourceCore already has these rows; ordinary local compilation captures
only the rare no-body cold declarations. A transient boolean activates the
existing definition-binding walk, with its full traversal prepaid once. Each
actual definition looks up only its matched row; the retained owners/effects are
read through their existing source image and configured-owner checks. The join
validates canonical function identity plus the actual associated trait member
scope or resolver-owned foreign block. It adds no node-to-definition map, body,
intrinsic, const, unsafe, import or call permission. Ordinary declaration and
call checking still reject missing implementations/bounds, free prototypes,
invalid foreign generics/ABI/result, unsafe calls and const calls.

The existing producer pays row/map/owner retention and failed attempts. Source
rows are not recreated, and repeated cold words do not capture the owner again.
Measured layouts remain ColdState 16, AnnotationState 64, Header 80, Owner 56,
Headers 120, Declarations 7120 and Code 304 bytes; no hot or wire field grows.
Private bounded-noise/retention, foreign-owner removal, wrong scope, foreign
source-image replay and trait-owner scope controls compile under all-target
metadata; execution remains for full CI.

The new PA31 packet freezes 20 inputs before candidate observations: 8 positive
and 12 negative, all checked in Rust 1.98 metadata and object modes. A genuine
no_core companion uses two real external C functions; both Rust and CPPGM native
observers print `cold-foreign 28 36 2`. The counter proves two actual calls.
Reference-v1's seven unrelated observer link attempts are retained as harness
setup failures; v2 preserves every source/prediction byte and links only the
actual native companion. No invalid source is relabeled as an accepted gap.

The earlier 27-input cold packet remains unchanged. Its two no-body gaps are now
positive, leaving three actual const/async body requirements in that packet.
Source advisory transport and broader source-parent completeness retain their
existing explicit fences. No whole PA, original-core or private-execution claim
is made by this local checkpoint; Parent owns joined integration and full CI.

Final qualification: obj/parallel/source-transport/cold-prototypes/final1/receipt.json.
Format/audit, all-target/all-feature strict metadata (105.54s), both new public
controls (14.71s), old cold and Word controls, actual CPPGM native, CLI build,
282 harness tests/457 maintainer roots/layout and pinned upstream 127/Reference 126
checks pass. Gates 2/3 retain the earlier overly narrow cause assertion against the
correct existing missing-definition error; no candidate status was changed.
