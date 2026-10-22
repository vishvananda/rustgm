# Caller-paid associated initializer premises

PA25 remains planned. This checkpoint closes the unchanged Rust-positive
`tests/spec/open-type-initializer-context/paid.rs`; it does not close unused
generic constant patterns or conditional nested outputs, nor generate oracles.

A source computation can instantiate an initializer with caller-owned lifetimes.
Its selected impl's declared trait premises previously reached a lifetime failure
even when the caller declared the needed relation. The new route first proves
the exact selection and raw implementation debts under that actual caller's
input header. Only a successful solve permits declared-premise specialization.
The actual initializer result, original and instance proof, const eligibility,
escape checks and ordinary concrete execution gate remain required.

## Ownership and proof ordering

Body dispatch borrows the exact matching Contract header. Prepared pattern frames
lazily retain an indexed original input header and keep their existing scope,
reservation and dispatcher across suspension/error. Source alias equations are
goals, checked once under these inputs before their existing identity fold is
used. A one-way outlives assumption cannot justify an alias equality.

The source-only dispatcher callback borrows one selected proof before it joins
the caller's append-only obligations. Raw declared-premise selection retains all
residual lifetime and normalization debts. Its proof and the fixed caller opening
must solve against actual declared/implied inputs; body obligations and selected
initializer requirements never become caller assumptions. The proof payloads are
then retained in their original order. No growing dispatcher prefix is scanned.

A private paid record binds the catalog epoch, complete caller context, selected
Instance and validated availability. Both cold and warm source uses revalidate
these identities. A warm selected target with no exact paid receipt requests a
fresh selection; no positive paid cache skips this solve. A source handle alone
still uses required-premise specialization. No executable or callable permit is
minted by a paid record.

Selected pattern types pass the unchanged final shape gate after entering their
actual parked caller namespace. Body uses close original aliases before opening
selected lifetimes, preserving the static control. No ownership test or ordinary
Associated concreteness rule is relaxed.

Lifetime-only original functions need no fabricated call argument list. Their
anonymous inputs retain the full InlineInstance and original Contract slots,
bounds and debts. The helper validates the real function body, canonical item,
inline owner and exact original binder. It folds the canonical signature through
the same early/late relocation used by registration and requires exact stored
callable-header identity. Missing, zero-early or changed same-arity headers reject;
general callable selection remains strict. This relocation is an identity check,
not a substitution of original proof lifetimes or an execution permission.

## Focused evidence

`obj/parallel/pa25-paid-premises/hardening-second/` passes strict all-target and
all-feature release Clippy, architecture/file audit (1,625 files, zero findings),
34 distinct private tests, and exact legacy independent-selection/lookup suites.
The final accounting follow-up repeats the focused proof/source/legacy checks in
`accounting/`, including nominal-preparation work on success and error.
The six new positives pass types/constants/LowIR and native value checks. Six
independent Rust negatives reject constants/LowIR at actual unpaid input proof;
unpaid pattern/inline bodies also reject types. The unchanged original open-Type
matrix passes its ten positives and two negatives, including native positives.
All sources are qualified with pinned Rust1.98 full object emission.

The private matrix covers foreign epochs, full contexts, availability, selected
arguments, changed publications, forged source/header/binder, disjoint reservations
and restoration. Repeated-root work at widths 8/32/128 is 938/3578/14138; distinct
roots use 1309/5221/20869. Retained debt storage is 256/1024/4096 and 384/1536/6144
bytes, respectively. Dependency work remains included. These are exact logical
operation/retention counts, not wall/RSS claims. Failed consuming lifetime solves
conservatively charge their entire existing 1,000,000-unit allowance because that
solver currently exposes exact successful work only.

Common layouts remain Target 28, Request 56, Code 336 and Dispatcher 912 bytes. The
new borrowed optional input header is 72 bytes; source-only BodyCaller is 344 bytes.
Pending pattern Frame is 1200 bytes (one new 8-byte lazy header pointer). Seal is 36
and its containing use row 72 bytes; existing size-based catalog accounting covers
that growth. Computation Catalog is 216 bytes with no new retained field. It is
distinct from the unchanged 200-byte execution Code Catalog. Ordinary constant
requests acquire no caller row or input-header allocation.

The earlier failed probe receipts remain. They include wrong pre-transition
shape ordering, static alias ordering, different signature namespaces, a test
counter initially counting a dependency as another root, and strict/audit fixes.
The unchanged unpaid open-Type source still rejects at its lifetime debt; its
new earlier diagnostic explicitly calls that a declared lifetime premise.

Full active text comparison, immutable binary/source-map identity and publication
prerequisites are recorded by the closing tail. Combined full release, generated
PA13/PA16 reference prerequisites and performance remain integrator-owned.

## Frozen handoff and central review

`qualified-final/identity.json` pins candidate SHA-256
`0c5eaa491f0c7a4b503ea24560b3a66488b265187d2418ff29c5cab64f282690`
and the 932-file production map SHA-256
`f20e2c71e38082f4a3ec8f96e0999e605fdcb6ba71f648c58a253806c04ef630`.
The independent changed PA20 negative still rejects a full Rust object; its paid
pattern opposite accepts. All 36 new source-view outputs are byte-identical
before/after the accounting-only edit, retaining the preceding native evidence.

The immutable `005bec4f` PA1-23 archive and previously exported 467-case PA24 tree
produce 8,255 comparisons: 8,247 exact and eight PA20 differences retained for
central review. Every other stage is exact. Relative to c2, six inherited byte
deltas remain identical; the inherited `trait-lifetime-proven` type view changes
its canonical/opened dispatch arguments and adds one aggregate type. A new
negative `projected-caller-lifetime-debt` still rejects at byte348, now at earlier
caller alias-equality proof. Its status is unchanged. `qualified-final/review.json`
retains all expected/actual hashes. The second comparison confirms exactly these
same eight differences; it is not a claim that the active suite passes. Central
qualification must compare the merged implementation against current public
references; no worker-base failure is automatically grandfathered.

The 89 Python harness tests and registration of 336 roots pass. Layout still
stops at the explicitly pending PA13 generated-reference receipt; PA13/PA16 oracle
prerequisites remain with the integrator. Upstream checks pass. No generated
reference, roadmap, assignment activation or push belongs to this handoff.
