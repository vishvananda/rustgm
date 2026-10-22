# Source binding reservation foundation

This local checkpoint adds explicit source-provider configuration and reserves
actual configured language-item and intrinsic declarations. It does not yet
implement semantic sizing validation or layout-intrinsic execution, and does
not claim an upstream core build. Range iteration remains the independent
`1034c8ab` checkpoint.

`--source-core NAME@DISAMBIGUATOR` requires the matching complete current crate
identity, actual `#![no_core]`, and explicit supported internal feature requests.
Ordinary APIs use a typed resolution context. Source mode suppresses bootstrap
preludes and synthetic preparation; real source definitions keep their IDs.
Both resolution pipelines reserve enabled claims after cfg and expansion, using
one attribute-owner map. Language items are singleton declarations; intrinsic
kind multiplicity is allowed across separate real DefIds. No reservation is
semantic or execution permission.

The parser permits bodyless functions only where direct intrinsic syntax or
current/enclosing configuration can require them. Plain and unrelated-attributed
module prototypes retain their original syntax rejection. After configuration,
ordinary surviving module/impl prototypes reject; source-mode prototypes require
the actual enabled indexed intrinsic claim. Intrinsic claims on associated
functions reject. Configured attributes are decoded from retained typed records,
without reparsing source or inventing owner nodes.

The approved next consumer uses distinct Sized/MetaSized/PointeeSized identities,
source-owned default clauses, checked declarations, and ordinary trait selection.
`rustc_deny_explicit_impl` must be retained as an actual source restriction:
bare lang declarations permit some explicit impls on the pinned compiler.
Explicit `T:MetaSized` or `T:PointeeSized` suppresses implicit Sized; PointeeSized
does not imply MetaSized. These are independent observations, not permissions
in this foundation. A Rust-accepted header outside the first supported provider
profile will remain an explicit unsupported boundary, not a Rust-invalid case.

## Qualification evidence

Evidence is under `obj/parallel/pa31/source-bindings/` in this worktree. The
independent source pin is Rust commit
`88d9e12ae178fab0fb5cc050a94da85685d449ea`, Rust 1.98.0, edition 2024.
`qualified-v3` settles 45 source outcomes: 19 accepted, 25 clean rejections,
and one retained host ICE for a direct runtime layout intrinsic. Thirteen
C-linked observations in the inherited `qualified-v2` execute layout constants.
An intrinsic fallback body is allowed and remains ordinarily typechecked;
an intrinsic cannot be reified to a function pointer.

`qualified-v4`/`qualified-v5` preserve additional parser and sizing hypotheses,
including incorrect original predictions and two further PointeeSized host
ICEs. None is represented as a clean language rejection. Pure cfg-disabled
module functions, associated functions, whole impls, and nested cfg_attr
intrinsic declarations are accepted independently. All six `qualified-v6`
sizing-default outcomes match their declarations. A parameter explicitly bounded
by PointeeSized may admit an extern type, while MetaSized rejects that type.

`foundation-v3` passes all-target/all-feature checking and five private tests
covering both pipelines, wrong provider identity, no-bootstrap mixture, disabled
and macro-generated claims, duplicate ownership, configured prototypes and
ordinary cold owners. Measured Package is 272 bytes and Resolution is 2160,
each adding one optional pointer to this base. Source context is 72 bytes,
reservation catalog 192, declaration 20, intrinsic record 24, and Parser 432.
The catalog/context are absent for an ordinary scalar source. Declarations,
Body, Type, value and call records are unchanged at this foundation.

The first strict tail found four feature booleans and two placement findings;
feature requests now use a typed bit set, the spliced-item validation has its
own grammar helper, and private tests use their owning reservation filename.
Original failed receipts remain intact. `foundation-strict-v3` passes strict all-target/all-feature Clippy, all five
reservation/parser tests, the 1,806-file audit with zero findings, layout and
suite inventory. Full combined release, active-text qualification and performance
remain pending; this is a declaration foundation, not the completed consumer.
