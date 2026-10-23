# PA31 fixture ownership

Planned: no graded manifest or generated reference outputs yet.

`spec/library-stability/` retains 31 independently qualified local/client
hypotheses and two providers. Its `contract.json` binds exact sources, declared
outcomes and original evidence. Eight clients executed with pinned rustc; local
library cases are metadata-only checks. Candidate implementation and maintainer
registration remain pending. See the [stability investigation](../../docs/pa31-library-stability.md),
including the retained module-stability confounder and corrected controls.

`spec/needs-drop/` retains independently declared source-provider inputs for the
actual `needs_drop<T: ?Sized>` intrinsic and its ordinary const-block wrapper.
The grouped maintainer `needs_drop_provider` tests discover these files directly;
this does not activate PA31. `contract.json` records the fifteen original
positive/negative source hashes and independent expected outcomes. `work/`
contains thirteen separately declared width/repetition/noise controls.

The small provider uses actual attributed declarations, not the complete pinned
Drop/Pin implementation. Full default-method, Freeze/drop_glue and conditional
destruction support remain required. Public generic/trait metadata and executable
entry readiness are distinct from checked LowIR/native kernel qualification.
See [the source-query review](../../docs/pa28-source-needs-drop.md).

`spec/drop-glue/` retains exact independently declared helper/cleanup sources and
C observers. Its contract separates the initial runtime family and semantic
opposites from still-required const and extra-role contexts. The grouped
`glue_provider` maintainer tests use the runtime inputs directly; registration
does not claim complete Drop/Pin, const or unsized support. See
[the destruction-helper review](../../docs/pa31-source-drop-glue.md).

`spec/source-freeze/formation/typed-depth-*.rs` retains six byte-identical
copies of the frozen whole-caller alias-depth inputs (depth 8/32/128, with
0/128 unrelated nominal declarations). Their explicit hashes, source profile,
independent observations and required native 7 outcomes are recorded in
[typed-depth-contract.json](spec/source-freeze/formation/typed-depth-contract.json).
The private `typed_sizing_work` controls measure actual caller inference and
canonical work plus retained proof-map storage; the grouped `provider_traits`
native test executes every copied source. This is development coverage, not a
PA31 manifest or whole-core acceptance. See the
[source-sizing review](../../docs/pa31-source-sizing-reuse.md).

Required families: lang-item identity and signatures; intrinsic declarations and const eligibility; builtin expansion and capability answers; duplicate and forged bindings; panic-abort runtime boundary.

Follow the [owning brief](../README.md) and
[library bootstrap plan](../../docs/library-bootstrap-roadmap.md).
Declare independent outcomes before creating text references. Register every
input in a nonempty `manifest.json` when the stage is ready for activation;
no undiscovered fixtures or empty-suite success. Preserve upstream checksums,
configuration/build provenance and linked native output evidence.

`spec/unsizing-roles/` preserves actual Unsize/CoerceUnsized source declarations,
independent goal/opposing contracts and dyn requirement width/caller controls.
Grouped `coercion_provider` tests qualify the bounded source-role family while
retaining valid unsupported nominal/header sources as required positives. This
is a prerequisite for runtime unsized cleanup, not cleanup completion. See
[the source unsizing review](../../docs/pa31-source-unsizing.md).

`spec/source-const-payment/` retains declared const-impl overlap/body legality and
intrinsic-versus-ordinary-name controls. `work/` contains exact copies of the
previously independently qualified reached-call, caller-definition, full-instance
and unrelated-definition inputs. The grouped `provider_traits` source/native
tests and private `reached_work` controls consume them; the Rust-positive
conditional intrinsic stays explicitly unsupported. No references or PA31
activation are implied. See [the per-use payment review](../../docs/pa31-reached-const-payment.md).

`spec/drop-glue-slices/` retains the original ZST slice helper and14 independently
settled runtime/borrow controls. Grouped `glue_provider::unsized_elements` tests
own its focused source/native coverage. The worker slice implementation is focused source/native qualified; combined
publication is separate. Dyn, owned Box and CTFE remain required work.
See [the slice-helper boundary](../../docs/pa31-slice-drop-glue.md).

`spec/source-header-transport/` records source-provider/client hypotheses for
actual sizing, Copy/Clone/default-method, Destruct/Freeze and receiver transport.
Its historical hypothesis disagreements are retained separately from corrected
Rust/native controls. This preparation does not enable generic source-role
metadata or source-sysroot client policy; see the scoped interface review.
