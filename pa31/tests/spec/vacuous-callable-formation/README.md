# Vacuous callable formation binders

These source-core reductions retain core DropGuard's `F: FnOnce(T)` bound.
`new.rs` uses parenthesized callable sugar; `angleout.rs` spells the argument
tuple and associated Output. Both are accepted and print `41 59` through
`observe.c`. `rejected.rs` drops a guard in a const function with only an
ordinary FnOnce bound; rustc rejects its unpaid const Drop requirement.

Every input and predeclared outcome is in `outcomes.json`. The shared
no_core prefix retains Rust's MIT/Apache-2.0 attribution; see root NOTICE and
third_party/sources.json. Nonempty higher-ranked binders remain quantified.
