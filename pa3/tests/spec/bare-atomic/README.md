# Bare atomic target facts

The fixed x86-64 course target includes bare `target_has_atomic`. Its complement
is false. Width values 8/16/32/64/ptr remain true, width 128 remains false, and
bare `target_has_atomic_primitive_alignment` remains false.

The manifest declares those three predicate results before text generation.
Independent pinned Rust metadata checks use constant-array assertions in both
stable and explicit bootstrap sessions. Stable Rust omits the bare flag;
bootstrap sessions expose it. The course already includes bootstrap load/store
facts and now consistently includes the bare atomic fact for this CAS target.
The compiler library reads neither that host environment nor rendered Rust output.

These are target facts, with override attempts rejected by the existing cfg
option parser. They grant no feature, intrinsic or unsafe permission. PA8's
paired controls retain real source/macro authority for load/store predicates.
