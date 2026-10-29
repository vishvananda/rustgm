# Captured source intrinsic calls

These authored NoCore reducers retain the genuine source intrinsic declarations
and the checked generic slice accessor used by the Debug array observer. The
foundation is shared with the source Location and wrapping-offset contracts.
They do not replace original core methods or supply an original library artifact.

A FunctionItem keeps a function binder around its signature roots. An opened
Body Call already stores its input/result roots in the caller. The borrowed
Formatter method exposes the difference with inner `&str` captures: the actual
argument has bound(0,1), while the item signature has bound(1,1). The old scalar
control remains valid. The immutable old kernel rejects the borrowed case;
Rust independently accepts it. Readonly canonical lowering maps both item roots
exactly; caller signature validation preserves full flags, arity and root IDs.

The const reducer requests the real inherited library feature and annotates its
defining const method. Earlier missing-feature/const-stability predictions remain
in maintainer evidence. Independent Rust metadata/object/link/native and candidate
LowIR/object/link/native all succeed and return `1`. No pointer is dereferenced
after wrapping. The SourceIntrinsic registry, unsafe/const/current callee proofs,
original source identities and metadata fences remain authoritative.

`contract.json` binds the final source bytes and outcomes. Focused controls also
reject wrong regions/types, own binders, safety/convention/arity changes and failed
work payment; shared nested DAGs scale without interning or retaining permission.
Whole original Debug/Formatter, source transport and parent assignment release
requirements remain open.
