# Source Unsize trait-object upcast goals

Ten authored reductions use the existing independently checked PA31 Unsize,
PointeeSized and cleanup foundation, including its pinned source-derived headers.
Expectations are declared in manifest.json and cases.tsv before implementation.
The principal must be an intrinsic bound of the source object; an unrelated
ordinary implementation does not authorize a trait-object upcast. Retain exact
associated equalities, auto markers and outlives debts. The actual-value case
additionally exercises real source reference CoerceUnsized and dynamic dispatch.
Quantified-object and nominal-tail Unsize continuations remain required.

The first custom auto-marker forms remain frozen as two required followups;
Rust qualifies them, while the current source object profile requires a
represented marker. The active erase/add controls use the genuine Send role.
The DispatchFromDyn prerequisite correction and initial independent errors are
recorded in the manifest and maintainer observations. No oracle was blessed.

The foundation is copied from ../unsizing-roles/object-goal.rs. Its source-role
headers derive from Rust 1.98 commit88d9e12ae178fab0fb5cc050a94da85685d449ea,
core marker.rs, ops/unsize.rs and intrinsics/mod.rs; Copyright The Rust Project
Contributors, MIT OR Apache-2.0. Case tails and observers are authored reductions.

All24 metadata/object observations match independent Rust, and the supertrait,
associated-type and actual reference-dispatch observers return7 under both
Rust and the frozen CPPGM backend. The source supertrait-diamond work invariant
passes at8/32/128 with8192 unrelated canonical types and no work change. Existing
43 source outcomes and seven private formation/lifetime/work controls pass.
Target-quantified objects, nominal-tail Unsize and original library artifacts
remain required. No source-client export or assignment activation is implied.
