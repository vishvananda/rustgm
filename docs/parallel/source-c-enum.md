# Explicit fieldless enum C transport

This finite PA30/31 checkpoint follows source cursor commit 7118818f4. It closes
the original Rust-positive cvoid.rs by-value C echo with its original source and
observer unchanged. The separate constructor/move/cast observer is retained.

One shared Layouts.fieldless_enum_scalar query checks the real nominal Enum,
an explicit integer repr, ordinary Rust/C representation, no packed/custom
alignment, prepared integer tag at offset0, exact primitive size/alignment, and
complete source/physical variant coverage with zero fields and equal discriminants.
It returns the primitive storage type. It supplies no enum validity, Copy, Drop,
const, lifetime or source-call permission and does not create a zero-excluded
Carrier. Existing Context representation, C Plans and nested composite caches
consume that same paid query; no new persistent cache, body/type record, metadata
schema or backend behavior is introduced.

The existing ScalarObject paths already preserve actual object slots/snapshots
and transfer exactly the selected leaf type at Rust/C parameters, calls and
returns. Construction still writes the actual selected variant's tag bytes.
Source types and declaration/variant identities remain nominal; scalar storage
does not become source type equality. The query's charged variant walk is linear,
and existing warm signature plans do not rescan layouts.

Independent sources were frozen before implementation. All twelve signed and
unsigned widths, including i128/u128, passed their real C observer. A second
observer covers a nested C struct, direct import, function-pointer callback and
closed const-generic enum. A third executes an actual nominal Drop effect.
The initial Drop fixture omitted its method stability annotation; that setup
failure remains beside the separately frozen corrected drops2 input.

The focused gate is obj/parallel/source-transport/cenum/gate1/receipt.json;
strict format/audit/Clippy is static2/receipt.json. Four regular tests include
exact typed scalar plan/nominal identity checks, six meaningful source rejection
causes, bounded work and separately named valid-profile requirements. Three
native observers match independently qualified Rust:

| Input | stdout |
|---|---|
| widths | 1 -3 65535 -3 4294967295 -3 18446744073709551615 -3 1 1 17 -3 |
| calls | 1 73 1 1 513 |
| drops2 | 9 |

The source move, distinct nominal identity, immutable place, repeated tag,
literal range and packed-placement opponents retain their ordinary causes.
Layout work is 24/72/264 for 8/32/128 variants; 32 repeated warm plans add zero
layout work. These counters establish bounded work, without an ABBA speedup or
general performance claim.

All seven original cursor/native observers now pass, including the formerly
blocked original c_void C echo. Runtime source/feature/discriminant/borrow
regressions, old Va controls, upstream verification and registration pass.
The source cursor const-definition/CTFE requirements remain open. Default repr,
payload enums, custom alignment and transparent wrappers around enums are
independently Rust-positive and retain explicit finite-profile C ABI requirements;
they are not semantic parity negatives. Full private/release checks remain CI.
No source standard declaration, body or attribute was changed to gain permission.
