# Source From role and numeric fallback

Status: required implementation work, independently qualified against pinned
Rust. This corpus does not claim RustGM acceptance or release PA31. The explicit
[contract](contract.json) records every source hash, required candidate outcome,
Rust object outcome and native assertion. The implementation order and algorithm
requirements are in [the plan](../../../../docs/pa31-source-from-plan.md).

The 23 sources cover real language ownership, renamed traits, ordinary and
diagnostic-only lookalikes, distinct diagnostic/language owners, explicit float
types, aliases, generic wrappers, late Self inference and both orders of shared
numeric obligations. Empty From traits remain valid: the language role does not
require a method with a particular name or return type. A completed blanket
implementation leaves ordinary f64 fallback intact. Four opponents reject wrong
arity, wrong declaration kind and duplicate language ownership.

All 19 positive programs emit pinned Rust objects and execute through observer.c
with their exact declared output. These are complete NoCore source families with
genuine sizing, receiver, Clone/Copy and, where required, Freeze declarations.
They grant no synthetic implementation proof. Source feature requests and
stability policies are explicit; advisory warnings are capped in the independent
bring-up profile.

original-trait.rs contains the unchanged From declaration and attributes from
Rust commit88d9e12ae178fab0fb5cc050a94da85685d449ea,
library/core/src/convert/mod.rs bytes22640..23206. The contract records whole-file
and fragment SHA-256s. This Rust excerpt retains the repository's Rust upstream
attribution and MIT/Apache-2.0 notices; all surrounding foundation, implementation
and observer code is authored qualification support. It does not replace the
original crate or prove its reservation implementation.

Oracle plan2f1ecc3def916a4ae635aae2e13e1ada140f7bcad0b9ae6252ad22eff9984810
and receipt5096f879c7a4b343ed55a35d7715ebbda49f63b873a00e434c2989db8757664b
retain commands, raw output, tool/source hashes and all prior fixture failures.
Generated objects and executables were pruned after success. No candidate text
references have been generated. Integrate the real typed consumer, then qualify
these same bytes through RustGM, checked LowIR and the locked native backend.
