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
control and genuine legacy Functions golden/Unknown test. Hosted run 40276199238 qualified all six nonignored groups on component
`5c0bc127925b2edf9a4571608e6af4c3392d143f`, with production and private strict
Clippy, formatting, metadata and the global architecture audit. The helper took
179.476 seconds within its 180-second work limit. Its source binding authenticates
2,720 compiler/configuration fingerprints and 31 fixture/data hashes. This is
source/typed-clause qualification; no native or metadata-intrinsic support is
claimed.

Measured sizes in bytes are BoundSyntax 56, Checker 10232, Declarations 7048,
Package 256, Unit 688, Node 36, Scope 32 and Definition 36. No allocator count was
measured. The implementation adds no field or owner allocation and reuses the
existing decoded-Syntax scan and generic-level map.

The qualified component is retained at `pa31-alias-self-sizing-qualified-v1`.
Merge `ed3261c32fb6514b9dfa4798386f78788ef31b53` combines it with public
`3cb7cfc5b8a80b3e0d2b3b00a4ecbea6dee1a824` without conflicts; the sizing
implementation, tests and fixtures are byte-identical to the qualified component.
That merged composition has not received a separate candidate run.

The successful primary receipt SHA is
`b8adf5a504064b5bcf5a67a6545e54f281d7c3822d1da7f90ef86661c673af27`;
the separate audit receipt SHA is
`39adec3d0dab3de965c70fd78e7609b812cf518c70398ddfb592acdf8893b5bd`.
The remote CLI SHA is
`2e4ed121176648a628b915197bdc3b0bbc1c1342f1667e3d2cad81c9dca7f3ee`;
it has not been hydrated locally. Earlier private-test type and semicolon failures
remain retained with their raw diagnostics.

Primary source revision: `88d9e12ae178fab0fb5cc050a94da85685d449ea`.
Predicates source SHA: `4992d01b843fa5af76e36389c727e7756c99cb36920af471552aaad49728791d`.
Bounds source SHA: `fb5cc2431386eacfb406acae4f3288330049f38c901f5adc889453037d611388`.
Observed binding: `98d4ed4b908382cb49baa5e9a16ae0a3f4d77835ea44495e20b903d2a8c6c159`.

The subsequent Pointee producer is separately reviewed; this change does not
register Pointee, Metadata or DynMetadata or implement metadata intrinsics.
