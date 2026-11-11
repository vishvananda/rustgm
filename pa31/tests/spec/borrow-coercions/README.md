# Borrowed-wrapper coercion requirements

These14 inputs freeze11 Rust object accepts and3 rejections before implementation.
Direct references and explicit dereferences are controls for implicit conversion
of local, temporary and borrowed wrappers, with shared/mutable targets and known
or inferred generic arguments. The negative cases retain actual shared-to-mutable,
wrong-target and escaping-loan requirements. A baseline type mismatch does not
qualify the escaping-loan rejection.

The ordinary hook requirements live in PA26; actual source language-item
counterparts live in PA31. Ordinary forms currently stop at core::ops imports.
Source forms preserve the exact existing const-Deref foundation prefix. Source
analysis accepts the3 direct/explicit controls; all7 implicit positives currently
report incompatible types. Complete checked-body/LowIR and genuine failure causes
remain required. CLI source-library transport is a separate limitation; use the
existing prepared root-builder API for the checked-body contract.

The native positive and shared C observer independently print
`borrow-coercion 23 41 1342` under Rust1.98/cc. It evaluates the wrapper once,
invokes real DerefMut before the next argument, changes23 to41, then invokes real
Deref for a shared read. Candidate RustGM/CPPGM execution remains REQUIRED and
unqualified. No negative oracle, empty success or cast substitutes for those calls.
Every input is in the manifest; no course reference or whole-PA release is added.
