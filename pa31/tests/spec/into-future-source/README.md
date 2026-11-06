# Complete IntoFuture source and ordinary conversion

original-into.rs and original-future.rs are the complete pinned Rust1.98 files,
unchanged. Every full provider wraps those exact modules, retaining the genuine
Future forwarders, IntoFuture associated equality and blanket `self` body.
original-mod.rs preserves complete future/mod.rs for the separate ResumeTy and
get_context requirement. Provenance is registered in third_party/sources.json.

The supporting protocol foundation is copied from the already qualified
async-iterator packet. Its Pin, Poll and Context declarations are authored legal
protocol dependencies. They do not claim the complete standard Pin/Context APIs.
In particular, the real Context contains Waker/LocalWaker, AssertUnwindSafe<ExtData>
and invariant PhantomData fields; that full closure, full NonNull dependencies,
actual derives and ResumeTy/get_context source remain required. There is no dummy
layout or generated-await authority supplied by this ordinary conversion slice.

predictions-initial.json and reference-v1 preserve the first failed setup: the
complete original Future's two documentation attributes need the real
core/lib.rs doc_notable_trait and rustdoc_internals requests. The corrected
provider retains every original attribute and adds those two requests. Three
wrong-target negatives are uncoded required-trait-method diagnostics, rather than
the initially recollected E0718. No candidate was observed before these corrections.

All21 corrected exact source outcomes match pinned Rust before candidate changes:
eight positives and thirteen real target/impl/output/body/const/unsafe/move/borrow
negatives. The native companion executes the original blanket and operand-dependent
custom source methods, retaining borrowed Future application identity. Its C
observer prints `23 23 47 59`. Object mode alone is not native execution evidence.
No const conversion, implicit await, generated poll/state, whole assignment or
original core artifact completion is claimed. Candidate observations remain
separate from this reference freeze.
