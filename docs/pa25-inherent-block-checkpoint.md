# PA25 const inherent implementation checkpoint

Qualified 2026-11-04. This clears LIB-142's const inherent-implementation owner
failure. PA25 owns implicit const functions and conditional calls, PA19 owns
method identity, PA31 owns source authority/library policy, and PA29 retains
authenticated transport requirements. Whole assignments remain planned; no
original core, alloc or std artifact is produced by this checkpoint.

## Semantics and authority

Consume the actual parsed const inherent impl and direct associated owner.
Its functions are implicitly const; redundant written const and async members
are rejected. Ordinary impl annotations do not grant function constness, and
nested functions do not inherit it. Validate the exact const keyword's existing
feature or macro allowance. Captured syntax cannot borrow a literal token's
permission. The ordinary local source owner retains the genuine request without
creating a compiler-item Catalog.

Const library policy remains independent of executable eligibility. Unstable
group policy can inherit to eligible methods; stable group policy does not.
Direct const annotations still require the pinned compiler's actual target
rules. Ordinary trait impl containers can carry const policy without becoming
const implementations. The original regular, const and default-body axes remain
separate. Public nominal exposure follows completed types and aliases: build
one prepaid nominal-to-public-inherent-member index, and visit each reached
nominal's members once. Private unreachable nominals grant no public exposure.
No public-type-by-all-impl rescan or source-text reparse is added.

Ordinary const functions now use the existing scoped call-context machinery
without requiring source-provider mode. A single optional global cold owner
holds ordinary rows; actual compiler catalogs retain their original pool.
Ordinary runtime calls stay absent. Actual declaration, body, full call target,
signature and conditional premises remain required. Caller-input accounting
uses the ordinary bounded partial attribute payer when no compiler Catalog
exists; this never completes source contracts or supplies execution permission.

Runtime lowering materializes conditionally const generic functions in runtime
mode. The later field-owner successor preserves unconditional const-function
and actual aggregate-constructor capability. CTFE uses its separate
materialization and reached-call checks. This permits
`Holder<Ordinary>::read()` at runtime while rejecting that same nonconst trait
implementation in a constant initializer. Original unused const bodies still
receive defining checks. The native regression exercises both paths in one
program rather than discarding the ordinary implementation.

Dense Node36 and Body456 layouts, Frame24 and Inventory112 remain unchanged.
The global Declarations record grows from7096 to7104 bytes for its one cold
pointer; all seven exact global arithmetic assertions are updated.

## Independent qualification

The explicit PA25/PA31 packet manifests cover51 pinned Rust1.98 outcomes,
including four upstream files preserved byte-for-byte with attribution and
checksums. Initial predictions are retained in earlier contract versions; the
corrected language v2/policy v4 manifests match observed Rust behavior. They
do not silently redefine a negative as success. The private test and production
CLI independently enforce every current outcome; negative CLI cases must not
succeed through a not-implemented diagnostic.

The final gate passes104 private and5 production checks in200.40s, including
strict Clippy, full formatting and architecture checks. A separate warm33.66s
native gate matches Rust's actual output `7\n9\n`. Source/bound/cfg/macro,
policy, exposure, budget failure and dense-layout controls remain covered.
Upstream attribution and harness checks pass. Ordinary and const-member LowIR
controls match the predecessor byte-for-byte. All fifteen frozen configuration
constants match SHA25686c36cf133e802357dedce4d7afadb0d5960235d8ad849a77556903a5e15f96d;
all six configuration source pins are unchanged.

The96 retained immutable A/A and paired ABBA observations compare ordinary and
32/128/512 constant-call inputs, verifying every output hash. Median B/A ratios
are0.936/1.067/1.024/1.042; A/A medians are1.016/1.121/0.986/1.016. Individual
blocks are noisy, and no speedup is claimed. Candidate medians are10.46ms
ordinary and35.90/83.58/308.45ms for the constant-call inputs. These are scoped
frontend measurements, not whole-library build or memory numbers.

## Original core and retained requirements

Immutable CLIfcaa05c0e99dcffc2dd230fdbde62a71823565cdbec7bfb6e81ccaade939b624
clears LIB-142 and reaches LIB-143 in17.86924s/375 opened sources:
`ERROR: file 42: 1399: unsupported library stability annotation owner`.
All722 original source hashes, target/options, Allow cap and finite source
limits remain unchanged: work14500624, attempted1GiB, retained64MiB.
The first-open map points to num/saturating.rs38's tuple-field stability
annotation. Confirm the retained owner and implement actual field policy,
inheritance, API uses and visibility under PA31/12/29; swallowing its unused
policy record is insufficient.

Authenticated imported const-block policy/calls, all remaining PA25 machine
requirements, original library declaration/body checking and real core/alloc/std
artifacts and consumers remain required. Original-source opens are not compiled
files or an artifact. Root works alone in the warm btrfs checkout with unchanged
five-minute, two-thread,16GiB output and150GiB free-space guards.

Full CI41845495353 on638524643 subsequently exposed five private and four
linking regressions after2662 passing private checks. Broader qualification is
failed, not pending: native/debug/independent checks were skipped. The successor
repairs trait-default policy requirements, ordinary source-use consumption,
imported API absence and unconditional const-body reuse. Its own qualification
must establish those repairs; this checkpoint's earlier focused evidence is
retained unchanged.

## Evidence

- `obj/qualification/fast/1793801056863893167/receipt.json`:
  `f2029756bb5c0a8b62bdc0ca3dafbb5fa63e1e927815cb7d9fbd9a359893aa41`.
- `obj/qualification/fast/1793801261645326294/receipt.json`:
  `eb4270ba34aa4de8a27c8b418dba57e73c1fa699c9b5498045b8296b7862a847`.
- `obj/parallel/const-inherent-v1/reference-v1/receipt.json`:
  `bbcee3c628670b338aa81ac81a3a748de34831099cc9c9ef1ad4336c0e9c5e22`.
- `obj/parallel/const-inherent-v1/qualification-v1/receipt.json`:
  `84577ebcb3fcabaf41d9cb465feb7f02ce106324749159fef79bf947c56e5531`.
- `obj/parallel/const-inherent-v1/measurement-v1/receipt.json`:
  `68c8ff0842bca9ac228457c759fb3e8dd878ff36ae2fc58dcdf2f411915c4efe`.
