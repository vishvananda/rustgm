# Structural inherent owners

The maintainer compiler now implements local primitive inherent ownership from
the actual crate coherence flag or the permitted member attributes. A crate name
does not establish that policy. Macro-local feature allowances cannot replace
the required global `rustc_attrs` request for member-based admission.

Candidate indices use structural head, name and context only to exclude work.
Applicability retains the full Self shape, const arguments, caller bounds and
visibility. Nominal lookup retains its existing fast path and declaration-time
overlap rejection. Structural owners publish the actual retained binding once
per owner/name/context while validating every declaration and body. Different
implementation owners remain distinct and ambiguous where required.

Receiver assembly follows actual autoderef steps. Autoref belongs to later
adjustment, rather than manufacturing implementation owners. For final array
unsizing, the compiler rebuilds the genuinely encountered reference prefixes
once with their original mutability and regions. Bare arrays acquire no invented
reference owners. Final unsized steps exclude by-value selection. This follows
the pinned rustc method-probe structure and preserves full-Self proof.

Ordinary local macro allowances use the existing cold source storage through a
distinct Local owner. They retain ordinary prelude behavior and gain no provider,
language-item or builtin-macro roles. Shared header publication, root-image and
owner authentication, literal/capture distinction, warm caches and budgets remain
the authority. Source-client admission remains closed to this new local profile.

[Focused integration](https://github.com/vishvananda/rustgm/actions/runs/40179084740)
passes nineteen exact private controls, formatting, the architecture audit and
strict private Clippy in 165.78 seconds. The same job passes all 212 harness tests,
upstream and audit-cache checks. The controls cover 66 immutable source cases,
ordinary and structural work counters, ownership opponents and existing source
provider/client guards. Independent pinned-rustc metadata/object observations
cover Rust 2021 and 2024; the candidate controls use the implemented Rust2024
profile. Original failed predictions remain in the fixture contract.

[Eight native programs](https://github.com/vishvananda/rustgm/actions/runs/40180633685)
execute through actual LowIR validation and the unchanged pinned CPPGM backend.
They cover scalar/member policy, distinct scalar owners, generic arrays, pointers,
references, associated functions and constants. Matching immutable rustc runtime
observations are reused for identical source hashes and profiles. Candidate CLI
SHA-256 is `50dac790f4a1f83dd015a552d02a745b6174a24a3cff5413993642a272dd6098`.

Measured layouts are Source 184, Local 128, Contracts 104, Core 176, Client 80,
Package 256, Report 3040, Definition 136, allowance Session 16 and Context 16
bytes. Qualified method paths remain 20 bytes. A single cold counter charges
ordinary/contextual/structural name lookup, including failed attempts. Depths
8/32/128 have identical charged work with and without unrelated nominal buckets.
Ordinary programs allocate no new structural owner unless they use it.

This is local feature qualification, not a PA31 release or original core artifact.
Imported inherent policy and macro contracts, wider local unsafe-lint headers,
test harness generation and full crate builds remain required. Mixed duplicate
member kinds, first-versus-second duplicate runtime behavior and nested unsizing
do not acquire qualification from the accepted declaration/lookup observations.
