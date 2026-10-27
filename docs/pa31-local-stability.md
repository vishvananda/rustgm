# Local library stability candidate

This candidate follows the [stability investigation](pa31-library-stability.md).
Focused static, semantic, ownership and native checks pass. The
[combined checkpoint](library-stability-callable-checkpoint.md) records completed
release/native and immutable-binary performance qualification. PA31 stays planned;
this is not a whole-core build result.

## Ownership and first supported profile

One optional boxed, resolver-owned `Package.library` owns root library requests separately from
source language capability bits. Root requests are parsed once; unknown possible
library names remain deferred until configured expansion and name resolution
complete. A library declaration never sets a compiler language capability.
Nested `feature` attributes do not change root requests. Ordinary programs with
no library requests allocate no library inventory. Consumers borrow the completed
state through `Package::library_policy()`; they cannot publish or replace it.

The existing configured-owner attribute pass validates regular `stable` and
`unstable` annotations, retaining their actual source owners, feature names,
versions, issue numbers and reasons. Duplicate levels/arguments, malformed
versions and non-identifier feature names reject. Cached include templates are
provenance; only installed active syntax contributes library annotations.

The initial ordinary-function/module profile is extended by the source Location
consumer below. The same owner pass now validates actual local generic, const,
nominal, trait, impl and scoped macro declaration annotations. Regular and const
API policies remain separate entries in the same table. Const annotations require
staged_api, a real const function/trait/trait-impl target, and valid nonconflicting
arguments. Immediate associated members inherit an unstable const parent policy;
a const function may inherit its unstable regular policy as implicitly safe for
stable const exposure. A selected const impl checks its policy against the actual
trait identity. Stable or implicitly indirect const bodies check each actual
callee's API policy independently of semantic const-call, unsafe, body and trait
obligations. Same-crate API naming needs no foreign unstable-feature permission.

The source Location checkpoint still closes dependency availability and all
v0/v1/v2 stability publication. Explicit default-body, allow_const_fn_unstable,
const_stable_indirect and implied/renamed policies remain unsupported. The semantic exposure correction below checks hidden-public nominal types in
normalized public interfaces. This remains a local policy consumer, not a
publishable original-core artifact. Foreign availability and broader effective
visibility of opaque promises and generic defaults remain required PA31 work.

Module instances retain separate effective policies even when they share a
parsed source file. Outer and loaded inner annotations identify the same actual
module. Module policy follows the existing parent-first module inventory; lexical
inheritance follows the indexed scope forest. Unstable policy inherits, while
an explicit stable policy stops general unstable inheritance. Only actual API
owners receive policies: local bindings, primitives and synthetic modules do not
inherit one merely because they share an unstable root. Public bindings
are traversed once by scope identity, preserving actual reexport targets. An
import has its own policy independently of the declaration it names.

Both source binding reservations and library inventory are built before either
is published. Failure retains root requests and cannot turn a retry into the
ordinary fallback. Same-crate ordinary unstable calls require no additional
library opt-in; no API feature pays a trait obligation or grants const execution.

## Independent owner evidence

`pa31/tests/spec/library-stability/owners/contract.json` retains 36 frozen source
families, including loaded/shared modules, and their declared candidate boundary.
The first 28 hypotheses produced 32 process observations including two native
programs. Two original hypotheses were wrong and remain recorded: marking a
reexport target does not annotate its import; a truly private type in a public
signature is accepted with capped lints. Eight isolating followups all matched:
publicly declared types reached through signatures need their own stability,
and marked import/group/glob cases distinguish import from target ownership.
The original candidate's five nominal limitations remain recorded in that
historical contract. The semantic exposure correction now accepts the four
independently valid forms and rejects the independently invalid unannotated
public type exposed through a signature.

Original owner results SHA-256:
`1adcbd583fd96369ab4c1ae8443f7ee4efbcc68071757b4ce14a07fc42f9821a`.
Followup results SHA-256:
`cf1cab06ac31d84757889c74461a14cc493429831da355577714050bb620fb64`.
These are independent Rust 1.98.0 observations. The earlier 33 API/macro sources
and their original outcomes remain unchanged.

The frozen `owners/contract-v2.json` retains all 36 original sources and adds two
explicit-helper runtime variants. The originals use a standard `assert!` macro
that this standalone candidate profile does not supply; they remain valid
unsupported inputs. Before candidate execution, both new variants passed pinned
rustc metadata, executable compilation and native execution (six observations,
empty output and status zero). Their independent results SHA-256 is
`613f27cf745141ed94d360ffe9e92a0b8b6e0eb5d2b41b90da700c26a842495a`.
No original fixture was rewritten to obtain acceptance.

## Work, storage and qualification

Attributes are consumed in their existing owner traversal. Effective owner
policy is computed once; public reachability uses a scope worklist, and each
annotated import tree is expanded once. The inventory uses bounded cold tables,
with one million work units and a conservative 64 MiB payload policy. These
charges are not whole-heap measurements. Hot semantic node/value records do not
acquire library fields. Actual layout checks measure Package at 248 bytes
(one pointer above its 240-byte base), Owner at 32 and NodeFacts at 80.

Nine independent scaling hypotheses are frozen under
`pa31/tests/spec/library-stability/scaling`: owner width and repeated uses at
8/32/128, and owner depth at 8/16/32. All nine pass independent metadata checks;
results SHA-256 is
`4eadd5246ebebe66ba3df969611c5ef485a09034a115e7f38f4d177524a477b9`.
The actual inventory counter observations are:

| Axis | Input sizes | Work units |
| --- | --- | --- |
| Owner width | 8, 32, 128 | 239, 647, 2279 |
| Repeated uses | 8, 32, 128 | 129, 153, 249 |
| Owner depth | 8, 16, 32 | 176, 240, 368 |

Successive work differences grow exactly with these input changes. They measure
the new inventory work, not whole-frontend work, allocation or elapsed time.
The combined checkpoint records the completed immutable-binary comparisons.

`local-development-v1` stopped at a filename-stem audit. V2 passed the audit
but strict Clippy requested a typed preparation result and a borrowed claims
map. V3 passed architecture checks and production type/lint checking but stopped
on one integration-test semicolon. V4 is the exact test-only recovery; it retains
V3's audit only after checking every source hash and that one-character delta.
V4 passed static checks but exposed a real ownership mismatch: module roots retain
their attribute wrapper, while the existing claims pass indexes its underlying
owner. V5 follows those retained wrapper edges with charged work. It also restricts
inheritance to actual API owners and adds the independently checked helper variants.
All original failures and admission refusals remain in their evidence directories.

`local-development-v5` passes formatting, the 2,120-file audit (zero findings),
all-target strict Clippy, harness checks (164 Python tests, 373 roots in 13
binaries), upstream verification, three frontend tests, seven private tests and
one native test executing both helper programs. The frontend table covers 38 owner
families and ten earlier API controls; explicit unsupported outcomes remain
distinct from language rejections. Private checks cover exact import/target
identity, ordinary no-allocation behavior, failed publication, exhausted work and
storage budgets, and the three growth axes.

Evidence root: `obj/parallel/pa31-library-stability/local-development-v5`.
Compiler/test/fixture snapshot SHA-256:
`7d0ed0fec7b7bbe2a01d49e9a67c823b2c3a6477219900826bf5373b8106ec7e`.
Checks receipt SHA-256:
`4014df9e1dba4885b939db417c65631e0812e8bb898cfd2f684ea59aee2e0b8f`.
These are focused candidate results; broad release/native/text qualification is
still required for integration and public publication.

## Source Location const-policy extension

The actual selected Location source, including its original regular/const policy
and macro allowance attributes, passes source checking, lowering and a native C
observer returning `31 7`. See the [source assertion milestone](source-assert-implementation.md#source-location-derive-consumer)
for exact source/LowIR hashes and the remaining caller-tracking/panic requirements.
The cold Inventory grows 80→88 bytes and embedded Library 120→128. One optional
24-byte ConstPolicies owner holds direct declaration IDs only when needed; the
Policy/Level sizes remain 48/24. No-const and ordinary no-library controls pass.
Both policy axes share the existing parent-first scope walk and bounded work/
storage charges. No Node, Body, Context, Package or Report field was added.

## Normalized public-interface correction

The regular and const declaration checks still run at the resolver boundary.
After semantic declarations and alias normalization complete, one temporary
interface visitor checks actual canonical type and declaration identities. Both
phases use the same resolved public-binding/module traversal, so public aliases
retain their real targets. Private nominal declarations do not acquire a missing
stability error just because a public function names them. Public nominals
reached through a signature do require their own policy. Unrelated types in a
private module and types mentioned only in function bodies do not become APIs.

The semantic pass shares one visited set across all interfaces. It follows
canonical signatures, explicit predicates, complete arguments and public fields;
it does not reconstruct source paths, expand function bodies, grant a trait
proof or publish metadata. The design charges existing cold inventory storage, including owned policy
strings, before temporary allocation. Root review found that the qualified
predecessor invoked this charge twice consecutively; root's single-charge
correction and refreshed combined counters are pending. Work and queue/index
growth are prepaid; failed attempts leave no retained completion/cache state.
Ordinary packages without a library inventory return without a walk or allocation.

Source `fa7c1ba982febb2e6e7ebb2469afed330cea1fc7` passed the two new private
semantic/cost groups and the existing ordinary-allocation and policy-scaling
controls. Formatting, global architecture and strict private Clippy passed in
the same 238.75-second bounded run. Production and combined frontend qualification
remain integration checks. The prior architecture-only filename failure remains
retained; no independent Rust observations were repeated or fixtures rewritten.

For 8/32/128 public functions sharing one normalized nested signature, the pass
observed 15/39/135 distinct types and 9/33/129 declarations. The twice-billed predecessor's work charges were 415/1011/3947, with conservative
logical storage charges 5204/13764/51332 bytes. These are retained historical
observations, not the corrected single-charge measurements.
Changing one unrelated hidden public type to 128 changed none of those work or
visitation counts; the larger preexisting inventory capacity was still counted.
These are bounded policy charges, not heap-allocation measurements. Actual records
remain Task 8 bytes, Inventory 88, Package 256 and Body 448. No persistent field,
profile, wire, role, callable or const authority changed.
