# Const Drop declaration field prerequisite

This checkpoint starts at bfef52bf6. Actual registered synchronous const Drop
implementations now validate every semantic declaration field in their original
impl environment, before cleanup demand. The qualified immutable 292c96f2 baseline
accepted all twelve new unused-impl controls, including the independently rejected
Copy-only, zero-length nonconst-array and nonconst-field cases. Those gaps are
closed through the existing destructor declaration owner.

The ordinary nominal runtime coverage comparison remains separate: only its
previously authenticated const Drop owner may compare MaybeConstTrait as Trait.
Field validation retains the actual Conditional Destruct goals and original impl
premises. It covers union fields too. Copy and array length zero do not waive
these declaration obligations; actual ManuallyDrop and reference semantics do.
Ordinary Drop, AsyncDrop and same-spelling non-role implementations are unaffected.

A transient aggregate::SemanticFields iterator shares one full-argument
Bound::joint fold and its memo between this validator and the existing cparts
consumer. It reads actual declaration templates, not physical region-erased shape
fields. The actual owner and canonical argument list are checked; callers retain
their ordinary nominal formation/schema validation and policy/effect decisions.
Variant visits, including empty variants, field visits, substitution and vector
allocation are charged. Empty variants allocate no substitution fold. The fold
is created lazily at the first field and reused thereafter. The iterator is 256
bytes and its transient Cost is 16 bytes on this host. No persistent Type,
Definitions, Body, NodeFacts, Code or wire field was added.

The field iterator is dropped before selection mutably borrows Declarations.
Normal scoped Nominals::Proof selects the actual Conditional Destruct predicates.
Its retained lifetime obligations start at the restored coverage proof frontier,
are appended contiguously in actual impl coordinates, and are consumed by the
existing destructor check_bounds gate. No declaration clone or independent cache
or permission registry is used. Diagnostics retain the actual impl source site.

## Evidence

source-const-owner-evidence.json binds compiler/test/fixture hashes and receipts.
The first guarded batch passed formatting, architecture audit and strict selected
CLI/integration Clippy, then hit its unchanged 230-second total limit during
public codegen after the joined baseline update. It produced no semantic test
result. Its continuation reused that completed codegen and passed four ctown
public tests, ctnext/ctdrop/vactx, registration and upstream verification. Genuine
native destructor observers matched Rust outputs 41 and 47; an additional
unconditional generic const Drop observer matched runtime operand 59.

ctown covers seven valid unused owners, seven invalid unused owners with located
field-premise diagnostics, two invalid-owner whole-bound companions, a structural
zero-array whole bound and four shared-field widths. Independent Rust references
predate candidate observation. Canonical work for one base field plus 0/8/32/128
repeated fields is base 407 with growth 28/124/508. This is bounded-work evidence,
not an A/B wall-time performance claim. Full private/release and joined original
core observation remain Root-owned CI/integration work.

Original uncertain positive predictions for Copy-only and zero-array const Drop
owners were corrected from E0367, preserving all original bytes and receipts.
Unused mutable/raw enum borrows similarly corrected to genuine E0493, while
shared borrow, conditional initialization, replacement and known array positives
remain independently frozen. No required Rust-positive case is called a parity
negative.

## Continuing work

The approved sparse variant producer is preserved separately in stash
37a1f97278019ed7563806089a8a9dc596e3d405 and described in
source-const-variant-design.md. It is not part of this production commit. Selected
leaf coordinates must remain plan-owned and distinguish Whole from OwnDrop;
physical live_type still governs drop-check independently. Its declaration
prerequisite is now executable, so selected leaves cannot hide an invalid unused
const Drop implementation.

Selected enum/precise cleanup, true interpreter cleanup scheduling and execution,
actual const Drop callees, initializer-owned eligibility and VaList const cursor
init/read/copy/end/early exit remain required. The original cldef/clcall inputs,
unit-result const payment controls, full unchanged standard module/macro and
provided Drop/Pin closure, and variadic transport fences are preserved. The Code
cleanup fence remains; no destructor was omitted or replaced with native fallback.
