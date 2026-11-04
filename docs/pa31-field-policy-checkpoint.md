# PA31 aggregate field policy checkpoint

Qualified 2026-11-04. This clears LIB-143 and repairs the regressions found by
const-inherent full CI41845495353. PA31 owns availability, PA12 actual fields,
and PA29 authenticated transport. Whole assignments remain planned; no original
core, alloc or std artifact is established.

## Ownership and consumers

Collect configured named, tuple, union and variant fields from their actual
resolved aggregate. One prepaid cold map retains each field source, parsed parent
and effective regular policy. Direct field policy overrides inherited parent
policy; fields and variants inherit stable as well as unstable aggregate policy.
Inactive fields stay absent. Const markers, duplicates and wrong targets reject.

The existing typed field producer authenticates each field/parent against the
map before publishing semantic fields. Unknown rows and copied parents are
errors. Construction, access, patterns, generics, union access and const reads
use their real existing implementation. Same-crate API uses remain allowed;
old metadata cannot export staged field policy. Actual imported consumers still
require authenticated transport.

Collection is one declaration/field walk with amortized map growth. Queries use
the index rather than rescanning annotations or siblings. Failed growth is paid
before publication; retained capacity joins the shared source/inventory meter.
Inventory grows112→120 and Library152→160 bytes for one optional cold owner.
Seven exact assertions are updated. Dense fields, Node36, Body456, Frame24 and
global Declarations7104 layouts remain unchanged.

## Const-call repairs and qualification

CI41845495353 on638524643 passed2662 private tests but failed five private and
four linking cases; later gates were skipped. This successor fixes the contracts:

- Trait defaults do not require individual const-stability annotations; typed
  exposure still checks actual inherent methods.
- Closed calls consume current ordinary scoped facts as well as provider facts,
  preserving actual source/body/target/signature/effect checks.
- Known nonconst calls reject before demanding a dormant imported caller-ABI
  inventory. API availability and abstract/default selection remain distinct.
- Runtime conditional functions keep runtime mode; unconditional const functions
  and registered constructors retain const capability. Actual inherited clauses
  classify functions, reusing prepared source summaries. Constructors never
  borrow a const-function effect-table row.

All24 independently declared Rust1.98 field outcomes match:18 accepted and six
rejected. The predecessor failed15 required positives. All51 const-inherent
outcomes remain. Final qualification passes130 private and7 production checks
in176.44s, including every prior CI failure, widened protocol/constructor controls,
strict Clippy, formatting and architecture audits. A separate33.09s native gate
matches Rust's actual `7\n9\n` in both programs. Earlier lint/filename failures,
the bounded combined-build timeout and constructor regressions remain recorded;
none count as passing evidence.

Upstream attribution and harness checks pass. Ordinary/member LowIR controls
match the predecessor byte-for-byte. All fifteen configuration values retain
SHA25686c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d,
and all six configuration source pins remain unchanged.

The96 immutable A/A and paired ABBA observations verify every input/output hash.
Median B/A is1.010 ordinary and1.025/1.026/1.047 for32/128/512 fields; A/A medians
are0.962/1.059/1.085/0.974. Candidate medians are9.54ms ordinary and11.01/12.44/17.97ms
for field inputs. The largest case shows4.7% median overhead for extra checking
amid host noise; no speedup is claimed. These are scoped frontend observations.

## Original core and remaining work

CLIfbfd6d1c2de849386ce724da7ecb05eacbe778efb74c18ecc7d589b59a4ca101
clears field ownership and reaches LIB-144 in24.52305s/375 opened sources:
`ERROR: file 53: 477: unsupported source intrinsic`. All722 original hashes,
target/options, Allow cap and finite limits remain unchanged (work14500624,
attempted1GiB, retained64MiB). Opens are not compiled-file counts or artifacts.

The first-open map points to intrinsics/simd/scalable.rs14's sve_tuple_create2;
the family also includes create3/create4/get/set. Confirm the retained owner and
inventory all active unsupported intrinsic declarations. PA28 owns intrinsic
contracts, PA41 selected-target/scalable operations, PA31 actual source authority
and PA29 transport. Declaration validity is separate from execution permission;
required reached operations still need real semantics. All original library
declaration/body checking, imported field policy, artifacts and consumers remain
required. Root stays alone in the warm btrfs tree with unchanged five-minute,
two-thread,16GiB output and150GiB free-space guards and scoped cleanup.

## Evidence

- `obj/qualification/fast/1793803842980927277/receipt.json`:
  `03ce9f0e4c8dc0466a7df164cefa580e790984d4f86c0ee919c86a49e29aecb2`.
- `obj/qualification/fast/1793804074155719885/receipt.json`:
  `efd591e3130740c193b63805c686ee75d84384ca32fc438b63b3cd435dc9caef`.
- `obj/parallel/field-policy-v1/reference-v1/receipt.json`:
  `b098c8f6deae928771795d91dac1ee2c49673a401939b0f5053648799f580076`.
- `obj/parallel/field-policy-v1/qualification-v1/receipt.json`:
  `ae434b4ca9a8b95f19452244d614f093780a2694e7841448e4d27b484350f23b`.
- `obj/parallel/field-policy-v1/measurement-v1/receipt.json`:
  `65c82e3fb10d4d5a628ce4e6f340c65b5a1ce4ba2aed3e95c3339cc951b0e584`.
