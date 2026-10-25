# Local source runtime-check policy

LIB-021 has a qualified bounded local source implementation. PA28, PA25 and
PA31 retain separate intrinsic, constant execution and source policy
responsibilities. PA29 policy transport, combined Test-removal demand, unchanged
core acceptance and combined CI qualification remain required. The CPPGM
vocabulary and backend are unchanged.

The qualified source component is `ca2f0a80`. Its staging on integrator base
`30264d8c` is an unqualified integration draft: the root unsafe-lint and feature
queries now share the borrowed checker, and their merged consumers still need
focused integrator qualification. The component observations below do not claim
that the merged tree has passed. A's body-numbering and defining-environment data
remain intact, with Provided publication closed.

The pinned Rust 1.98.0 source lowers the actual `ub_checks` intrinsic to a runtime
check operand. `rustc_preserve_ub_checks` suppresses only the `SimplifyUbCheck`
visitor, so ordinary optimization still runs. Without preservation the producer
freezes its session value. A retained operation can use the code generation
session when its actual generic or inlined original is transported; an ordinary
object call retains its producer's machine code. The thirteen frozen independent
native observations distinguish these paths. Constant execution returns true,
including when runtime UB checks are explicitly disabled.

`RuntimeCheckConfig` is an immutable compilation input, defaulting to false for
the course profile with debug assertions disabled. The driver accepts explicit
`--ub-checks=yes` or `--ub-checks=no`. The package retains that typed setting;
`cfg!(ub_checks)` reads it, and custom `--cfg ub_checks` is rejected. The checked
root reader owns the crate-only no-argument preservation request and the
`cfg_ub_checks` feature request. Neither a request nor a crate name supplies an
intrinsic declaration, body proof or operation permission.

A dedicated descriptor validates the actual safe const Rust function with no
parameters, generics, regions or predicates and a bool result. Constant execution
checks the fresh reached direct call, exact selected declaration and signature,
current body/frame, source/library const gates and actual fallback code. Repeated
calls pay again; no result cache supplies permission. Runtime lowering consumes
the actual checked containing body/call and evaluated callee before emitting an
existing LowIR bool. A cold catalog session retains `Frozen(bool)` or `Deferred`
as policy data and rejects a provider report from another profile. Imported and
migrated bodies reject missing transported policy instead of inheriting a local
catalog default. No new central Rvalue or per-Body, Context or Declarations field
is introduced.

Configuration evaluates each demanded predicate once and retains its typed
`CfgUse` with an existing effect/capture identity. Truth-irrelevant operands still
demand their gates. A predicate that disables its own owner still demands its
gate; an already inactive ancestor, later attribute or inactive `cfg_attr` payload
does not. Typed interpolation spans retain the original caller token provenance.
The expansion engine and completed report use the same borrowed `FrameView`
checker, with one lazy engine memo for repeated `cfg!` queries. A prepared
`cfg_select!` choice owns the single parsed replacement and is authenticated before
publication. Unselected branch syntax is parsed, while its body configuration
and nested macro expansion remain undemanded.

Source cache identity is a separate measured representation change. A source
incarnation owns an `Arc<OriginalImage>`, retaining immutable original bytes in
an `Arc<String>`. Physical module copies share bytes but receive fresh source
incarnations. Cached headers retain the image and continue checking the current
frame, header, owner, enabled state and attribute range. This prevents allocation
reuse from turning an unowned pointer and length into authority. Original text,
CRLF normalization, append-only semantic spelling and physical positions preserve
the existing public API. The separately qualified owned-source component
`4fdbc97c` measures SourceFile at 56 bytes, down from 72. Construction reuses
the input byte buffer and adds two small Arc allocations totaling 64 bytes; a
physical copy shares the immutable bytes and allocates one 24-byte incarnation.
Focused incarnation, CRLF, BOM, append and position tests preserve this contract.

The source contract is `pa31/tests/spec/ub-checks/contract.json`. The first seven
extra cfg-select observations preserved one failed acceptance prediction: its
unselected statement payload was invalid in expression context. That syntax
failure is not evidence of a UB gate. The original source, hypothesis and raw
observations remain retained; all three separately frozen expression-valid
follow-up observations matched. The thirteen native policy and fourteen original
demand/origin oracle predictions also matched. An adapted same-source oracle
matched all 48 commands, including six actual native profiles and the cfg and
callable opponents. Cross-crate policy observations describe the required future
transport semantics; they do not authorize imported operations in this slice.


The candidate's frozen functional-v5 run passed formatting, the architecture
check, all-target strict lint, fixture registration, upstream verification and
all 26 measured layouts. Its 1,519 unique private tests, 24 frontend functions
and 84 native contexts passed. The native contexts include the unchanged
selected-reference anonymous caller's exact `7 1` output, all thirteen early
readiness contexts and six local UB profiles. The wider private/native run was
already live when development validation changed to the five-minute focused
workflow; new edits do not repeat it merely because a binary changes.

A separate authenticated CLI-only continuation passed seven observations in
2.57 seconds. The default typed view equals explicit UB-disabled output; enabled
output differs. Plain, raw and valued custom `ub_checks` options and an unknown
UB option return usage exit 2. Earlier failed observations are retained:
`--emit-body-ir` honestly rejects the fixture's symbolic generic body at the
existing concrete-body inspection boundary, and the source-core CLI rejects the
expanded-crate view. The supported typed view checks driver configuration
without changing either boundary. The source intrinsic and runtime behavior
have their separate native and semantic evidence.

The cold containing-session record is two bytes within Catalog's existing
536-byte size. Config remains 72 bytes, Package 256, Report 3,032 and the macro
allowance session 16. Body, Code, const evaluation state/machine, lowering
Context and Declarations are unchanged. ConfiguredUses is a 24-byte cold owner
behind an eight-byte optional pointer. ConfiguredSyntax grows by eight bytes,
Engine by eight; Unit falls from 680 to 672 bytes after the source change. A
cached header retains its 16-byte owned image/header record. Ordinary predicates
allocate no retained gate-use owner; growth and warm mutation controls keep the
shared header walk and exact source identity observable.

Combined behavior with genuine no-harness Test removal remains a separate
integration requirement. C's 144 independent observations distinguish demanded
own/ancestor/live-sibling predicates from undemanded strict body descendants.
A future checked removal receipt must authenticate actual typed body ancestry,
source incarnation, effect/capture and token origins. A disabled owner flag does
not provide that receipt. Body-root inner attributes and header/parameter/
generic/result attributes remain demanded outside the observed domain. This
checkpoint does not claim that combined consumer is implemented.

Imported/migrated UB operations still reject missing transported policy. Broader
`check_language_ub`, overflow/contract checks and Miri behavior remain open, as do
valid generic early readiness, broader anonymous caller profiles and full
unchanged core/alloc/std acceptance. Development follow-ups use exact affected
controls under the five-minute fastcheck budget; combined release, broad text
and performance qualification belongs to the integrator and periodic CI.

The current merged source also passes 16 exact private controls, including
the shared unsafe-lint authority and existing Functions codec/fresh-client and
numbering-denial sentinels (246.81 seconds locally). CI run `40170400821` passes
the four affected frontend functions and PA7/PA8 text suites in a 23.87-second
warm fastcheck, all 198 Python harness tests, exact backend installation and
the backend smoke contract. The earlier filename ownership and function-size
Clippy failures remain retained. Native component evidence above is retained;
the complete merged native suite is delegated to periodic CI.
