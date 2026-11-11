# Borrowed-wrapper reference coercions

The2 ordinary C ABI programs independently execute through Rust/cc, checking
borrow-coercion23/41/1342 and borrow-guard23/41/1345. Their core::ops imports still
prevent candidate parity. Both remain REQUIRED POSITIVES. The shared C observers
also serve the qualified actual source programs in PA31.

All28 original inputs retain their exact bytes and independent outcomes. Every
input and C driver is in its owning manifest. The original draft inference
predictions were checked against Rust: unhinted statement/unit calls reject with
E0308; explicit-type scaling positives were declared separately. No Rust-valid
positive was inverted. See the [qualified implementation](../../../../docs/pa26-borrow-coercions.md)
and [earlier reduction record](../../../../docs/pa26-borrow-coercion-reductions.md).
Ordinary bootstrap, broader composition, source/library transport and every
parent requirement remain required. No new course oracle or whole-PA activation.
