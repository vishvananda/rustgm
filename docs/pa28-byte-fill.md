# Canonical byte fills

This development slice adds the canonical `core::ptr::write_bytes` and
`std::ptr::write_bytes` identity, append-only intrinsic index 21. Its ordinary
signature is `unsafe const fn<T: Sized>(*mut T, u8, usize) -> ()`; it has no Copy
bound. The existing intrinsic payload carries its selected callable. Function
reification and generic calls use that same identity, while const function-pointer
calls retain their independent rejection. No backend or public value schema
changes are required.

## Source contract and independent differences

The pinned library source is Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`,
[`library/core/src/ptr/mod.rs`, lines 636–704](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ptr/mod.rs#L636-L704).
Lines 648–653 require write-valid storage and proper alignment, including an
empty effective byte extent. Lines 691–703 define the Sized-only signature and
its language-UB precondition check. The helper's CTFE distinction is in
[`library/core/src/ub_checks.rs`, lines 105–120](https://github.com/rust-lang/rust/blob/88d9e12ae178fab0fb5cc050a94da85685d449ea/library/core/src/ub_checks.rs#L105-L120).
These are safety preconditions, not a promise of a runtime sanitizer.

The original independently declared rejection hypotheses for
`numeric-misaligned-zero` and `packed-field-zero` were disproved: pinned rustc
accepts their constant evaluation. `packed-field-one` also accepts. The frontend
retains the documented alignment requirement; the packed cases are explicit
CTFE oracle differences, not rejection parity. The numeric case reaches the
existing numeric-provenance boundary first, so it does not establish the
frontend's alignment rule. None has a valid-native claim.

Null and numeric ZST cases retain their exact source outcomes, separately from
the documented pointer-validity rule and missing numeric CTFE provenance. The
default `invalid_null_arguments` source lint remains required work; a numeric
address diagnostic is not evidence for it. No numeric/null native program runs.
The symbolic zero-count and ZST controls use address-only traversal; they retain
type, ancestry and alignment checks, without reading contents or granting writes.
Nonzero accesses still require live storage, bounds and the incoming pointer's
write permission. A separate shared reborrow never revokes the original alias.

The original `usize::MAX` spellings remain unchanged. This checkpoint's base
does not prepare that primitive associated constant and rejects with
`missing value resolution` before the operation. Independently declared literal
`18446744073709551615usize` counterfactuals test the actual ZST and overflow
semantics. The library preparation gap remains PA26 work; the original negative
overflow fixture alone is not evidence of arithmetic checking.
The original invalid-char native input similarly uses `u32::MAX`; its failed
candidate compile is retained, and a separately qualified literal spelling
supplies the actual native value comparison.

## Storage and runtime behavior

Constant execution checks the byte count against the target allocation limit,
then resolves one destination `TransferSpan`. A nonempty fill checks the existing
write authority and uses the root's exclusive byte backing. The backing is
converted once when necessary. One interval edit removes only intersecting
symbolic fragments, one byte slice is filled, and only touched initialization
mask words become initialized. Padding is filled too. Numeric bytes do not
invent pointer provenance, and partial word fragments must be restored before
typed demand or complete immutable publication. Invalid bool/char bytes may be
transported or repaired before a typed read; typed reads still reject them.
Ordinary writes and range aliases observe the same backing.

Work and temporary/retained capacity are charged before mutation. A pending
typed read forbids backing mutation. No source snapshot, per-byte provenance
rescan or whole-root Image version is created by repeated fills. Existing union
publication limits and unsupported storage families remain explicit.

The runtime shim validates the unsigned count before multiplication and pointer
indexing, then emits a U8-store loop using frozen LowIR vocabulary. Zero-sized
elements return without a loop; zero count cannot reach an external store. The
only byte value is the evaluated u8 argument. Neither path reads external
storage or creates a reference. Filling does not drop overwritten storage;
source arguments are evaluated once. Unsafe caller obligations still govern
runtime alignment and destination validity.

## Qualification

`pa28/tests/byte-fill-contract.json` declares each source hash, pinned-Rust
outcome, three frontend outcomes and valid native input. Original hypothesis
receipts remain under `obj/parallel/pa28/byte-fill/independent-first` and
`independent-additional`; literal follow-ups are separate inputs and receipts.
Native tests consume their exact source bytes and remain explicitly ignored for
ordinary CI. `dev/tests/byte_fills.rs` is registered once in `suite_memory`.

Private controls cover exact mask word edges, empty versus expired/shared
access, budget failure without destination change, local fragment edits and
typed shim effects. The original first source batch retains its two
MAX-spelling positive failures. The closing receipts in
`obj/parallel/pa28/byte-fill/closing-third/` qualify:

- 47 declared sources and all 141 frontend views, including explicit boundaries
  and precondition differences; 15 exact-source Rust/CPPGM native pairs;
- 49 independently qualified work inputs, 23 selected private invariants and
  40 total focused test functions, including the existing memory regressions;
- all 8283 active PA1–24 text cases on this branch, strict all-target/all-feature
  Clippy, a 1615-file audit with zero findings, layout, upstream and suite mapping.

At fixed type width/depth, byte extents 8/32/128 add 72 and 290 steps, independent
of tuple width 4/32/128 and path depth 0/8. The depth-8 surcharge is constant in
extent. Ordinary controls have no count-dependent cost; adding 64 unrelated
types changes neither steps nor peak slots. Repeating one-byte fills 8/32/128
times adds 816 and 3264 steps at root widths 64/512/4096; only initial conversion
depends on root size. Fragment-bearing roots with 8/32/128 words cost
10624/14720/18816 steps for 128 two-byte writes, reflecting logarithmic interval
lookup, with unchanged retention after the initial split. These are logical
work/storage checks, not a new throughput or RSS measurement.

The all-dev-Rust source fingerprint is
`c78837cd5c5085c49aba75d4e07f6b3139e2795607c4c28c6276d76a1ead5a3a`;
the production-only fingerprint is
`0573f73f83a08f4e3ef30261db064b8f7d4653012e85992f5e565ee4f0d5e4e6`.
The candidate SHA-256 is
`888e9463e54a73d9413fed04f9a1ecf1570d4977545d2d4e9fd37f06d9fd2cfe`.
Backend hashes remain locked. Later changes in this checkpoint are prose only.
This slice does not activate PA28 or replace combined release/performance
qualification.
