# Alias Self source sizing

The source candidate extends the existing bound pass in
`declarations/generics/bounds/sizing_defaults.rs`. An alias uses its actual
lexical Self and current prepared schema, never an associated trait owner or
an inference from the Self parameter's unsized flag. One existing decoded-Syntax
scan retains separate direct-RHS sizing bits for that exact Self; generic formals
continue to include their actual where-clause bounds, even when unused by the RHS.
Real role IDs distinguish Sized, MetaSized and PointeeSized from lookalikes.

Direct PointeeSized suppresses the alias default MetaSized. A where-Self bound
and an alias-mediated PointeeSized do not suppress that default. Direct Sized
keeps its explicit bound without adding synthetic MetaSized. Existing ordinary
trait/parameter contracts keep their exact behavior and legacy artifact bytes.
The existing imported role authentication and alias execution/export fence stay
in force. No bounds.rs, aliases.rs, owner representation, hot field or new map
is added. The role lookup is factored from the same existing implementation.

Pinned `collect/predicates_of.rs` treats trait aliases as TraitDef for defaulting;
`hir_ty_lowering/bounds.rs` only searches where clauses for TyParam contexts.
The lowerer's generic ?Sized machinery does not override HIR legality. New
independent run 40273407166 collects 22 metadata-only observations, no native
program, no infrastructure failure. Four positive status hypotheses were wrong:
both alias-RHS and where-Self relaxed bounds are rejected in both editions.
The original frozen plan, sources, raw errors and all disagreements remain
retained; candidate relaxed-bound rejection is unchanged.

The no_core provider controls inspect actual canonical alias applications and
typed clauses, distinguish unused formal identity from actual Self, reject wrong
source owners, retain the execution fence and cover ordinary absence/layouts.
The focused qualification also selects the existing explicit source-sizing
control and genuine legacy Functions golden/Unknown test. These six groups and
actual layouts are pending until hosted strict-first qualification; no candidate
source or runtime acceptance is claimed yet.

Primary source revision: `88d9e12ae178fab0fb5cc050a94da85685d449ea`.
Predicates source SHA: `4992d01b843fa5af76e36389c727e7756c99cb36920af471552aaad49728791d`.
Bounds source SHA: `fb5cc2431386eacfb406acae4f3288330049f38c901f5adc889453037d611388`.
Observed binding: `98d4ed4b908382cb49baa5e9a16ae0a3f4d77835ea44495e20b903d2a8c6c159`.

The subsequent Pointee producer is separately reviewed; this change does not
register Pointee, Metadata or DynMetadata or implement metadata intrinsics.
