# Complete local macro allowances: development handoff

Qualified worker source is `222b637a9a4adb8c1fa218ef8a857c6a22a24222` on
`pa31-allowance-sessions`; production is unchanged from `45c9a757` (the final
commit changes only the provenance scaling test). This is local declaration
and provenance support, not completed foreign unstable-API enforcement or
LIB014/PA31 closure. Generic scoped-macro admission remains disabled.

## Actual original-core progress

The unchanged original core probe now passes the prior
`#[allow_internal_unstable(panic_internals)]` at macros/mod.rs line 41. It stops
at **line 170, byte 6523**, the real `pub macro assert_matches` declaration:
`scoped declarative macro hygiene is not implemented`.

The compiler returned 1 without an infrastructure failure in 0.0327 seconds.
The wrapper returned 0, recording a frontier rather than accepting the crate.
All shared core files were authenticated before and after execution. The four
opened source files were lib.rs, prelude/mod.rs, prelude/v1.rs and macros/mod.rs;
the latter retains SHA
`9784131e43d255fed7a07ae18cbc72748eedbebe65dfcefeee999989289df30c`.
No source reduction, cfg change, edition change or library substitution was used.
The compiler is the worker's exact qualified debug source, not a claim about a
later integrated/public compiler.

Evidence is in
`obj/parallel/pa31/complete-allowances-whole-core-v1`:

| Receipt | SHA-256 |
| --- | --- |
| run.py | 80144d061e6f3714004bcf948fb1778a03f23bdc18b57f1162cf63e671e84bdd |
| binding.json | 1c8e78ccb0a6f79a7cb376e429c81fcba00b58bd9de61ae29b6ad7520d41da74 |
| result.json | fb12fac6b2860c8572417c4e3df13919e4d54ac7ef335ee8d7441571a137c502 |
| opened-sources.json | d567131f842357bda8105172a5bfd8ecf2c6fc072bdc56230bf97d4d5524af92 |
| stderr | 1d5fd7270845ce49fd0f2762e073681e6e34eee0f0ae9614f1164730ae8da7dc |

Resource token `aa1d9107415543ebae939210daa9a5ef` finished with returncode 0,
error null and its 16 GiB absolute cap unchanged. Scoped cleanup
`1792847381323547941.jsonl` completed, removing zero disposable files.

## Source and authority

Complete canonical identifier lists combine repeated attributes, preserve
unknown names as inert data, deduplicate membership, and retain every exact
source-lexeme/Attribute/Forest witness. NFC and raw identifier spellings are
separate from exact source authentication. Known `EditionPanic` membership is
consumed in both real definition and reached-frame checks. Presence of some
allowance does not skip a frame.

The immutable cold header is shared between the existing SourceCore entry and
its actual macro Definition. A borrowed public Report session authenticates
that header once and memoizes positive, empty, metadata-only and Plain negative
headers. Each warm query still checks the actual published identity and sticky
work/storage budget. It uses the raw token's single outer mark; it neither
unions ancestor grants nor grants literals' allowances to captured tokens.

Report reauthentication first authenticates the published underlying item and
exact Rc, then uses its retained attributed-wrapper owner for `checked`.
Bare-item substitution still rejects. The full source witness, final underlying
item and Rc equality remain mandatory. Cross-report same-numbered identities,
foreign Rc rows, source changes, absent warm rows and exhausted budgets reject.

`FeatureName`, `Policy::feature_name` and `MacroAllowanceSession::allows` expose
checked canonical names/provenance facts. They do not alter crate requests,
authorize API execution or bypass callable/const/trait proof.

## Independent observations

The original 21 hypotheses and additive seven list-data plus three Unicode
hypotheses remain byte-identical. Their contracts are unchanged:

| Contract | SHA-256 |
| --- | --- |
| internal-allowances-contract.json | 1cc6626e306b367d468e3db987cb71d069c5052d6372270837565a6c2f648f31 |
| complete-allowances-contract.json | d0159d58e7967a3324268f8392a2ae8f97a684f08c5357e3aa68ee8ba7e90951 |
| normalization-allowances-contract.json | 9a6eaebd1f87feddd73792a0c84fa3aeb2dc1a64c0cd446cd9a74ed2d2b5ec9d |

All **31 sources / 62 metadata-object language checks** matched the independent
predictions (22 accepted, nine rejected). All **22 actual native programs**
printed the declared `7`.

The first runner executed **100**, not its planned 106, observations: all 62
language checks, 22 links and 16 native executions. Six bare C links lacked
`core::panicking::panic_const::panic_const_add_overflow`, so those six native
programs did not execute. These are retained harness-link failures, not Rust
language rejections. The exact original source/overflow-checks=yes profile was
then compiled as staticlib with its real Rust runtime closure and actual
native-static-libs. That separate continuation passed 18 new observations,
including the missing six native executions. No fake panic symbol or disabled
check was used. In total, 118 observations executed, including the six original
failed links; there is no claim that the original 106 all passed.

| Evidence root | Receipt | SHA-256 |
| --- | --- | --- |
| complete-allowances-independent-v1/oracle | results.json | 1d0131cad9e6994182c564d47775d38a39e626e609abbf08d61b7e7c02461620 |
| same | complete.json | 16adc4f36212a455981cbca73a80f45f8871586947ea2380473259caa4ce8c7a |
| same | binding.json | 2e21baae7051ceacce22a501a9a0984ec67af9865653ff19fd329d6d6c9e85ac |
| complete-allowances-native-link-v1/oracle | results.json | e9055706cbd866da6ea9774ae6abc0a5abea5e1830ece02e5a8077a74a196340 |
| same | complete.json | d32b3e16111063231af0bf30435bd4e6d0346cfafb55ea3f48d485a000bfac65 |
| same | binding.json | e49cdd347bf034da8f33f1f464a13af296bbfde4cecb8c4c48245d022a375e1f |

Both runners used 8 GiB absolute caps and completed cleanup. Six generated .a
archives required a hash-bound retirement rename to a recognized Rust archive
suffix before storage prune; the map and both cleanup receipts remain retained.
The existing 168 independent panic observations / 35 native programs are
separately authenticated and were not repeated. External intrinsic/API rows
are independent evidence only; they are not candidate foreign-API support.

## Candidate qualification and failed recoveries

`obj/parallel/pa31/complete-allowances-source-v4` passed fresh format,
architecture audit, all-target/all-feature strict Clippy, harness and upstream
checks. One debug build supplied the ordinary CLI/library and test binaries.
The full nonignored private inventory passed **1393/1393**, including all 26
required cases (13 new, ten existing panic/repair controls, three layout
controls). Four frontend functions passed **43 source contexts**, and the native
function passed **34 programs**.

| Receipt | SHA-256 |
| --- | --- |
| snapshot.json | 9e14d8e0bb76ffd7ffa7264377af1adc4e69d729140a77a4828ac4a4518bda3c |
| checks.json | 048c2d9c36a2023690f22c0ceaa5268d76f9414df1ee91c3e6b10803bf74b373 |
| complete.json | 66fd5df3186bab32b1bdc956995122d9cdc2f5d34e87ec6e7db25c8f9aab7086 |
| inventory.json | ad097cd9921ac50c3e76878dfd73ab864ffaf221f1e275597d4dc09b92350613 |
| full-private.stdout | cfd007c5f5957be351681d05a9ad177e9d6f40cb5bbf04f252da14dffbd2d08b |
| full-private.stderr | 9c6070b7099ddc0989a791fda39b74531246823b094fe4d1a484e9d36a2a36c0 |
| frontend.stdout | 4a6d419f70e8a9507afe3c26a8f6e0896c07dc13c2351842df90e8965534818f |
| native.stdout | 0b5678c3510e5e9f6b88fa703365937ed3d9c5b181b57e50c40f79986a12f6f3 |

Token `934ad1e0afe143adb7294883b52d248b` finished successfully under 16 GiB;
default cleanup `1792847284620120405.jsonl` removed 68 disposable outputs.
The small qualified CLI pin is
`complete-allowances-source-v4/candidate/rustgm`, SHA
`2a8bf052b0d0d77d5d19ba740c7d8e1c105544f3a64efe45d741e50a6f48554f`.
Retire this pin after the integrated candidate supersedes it.

All failures remain separate. V1 stopped at six style findings before candidate
execution. V2 ran 1392 private cases: 1383 passed and nine failed. Eight exposed
the attributed-wrapper/item mismatch; one metadata site selector also included
a physical import alias. V3 fixed those real owners and ran 1393: 1392 passed;
the sole nested-source depth-128 scaling fixture exceeded the unchanged parser
grammar limit before its query. V4 uses a separately declared constant-grammar
forwarding chain, wrapping each captured expression so actual provenance depth
is measured rather than inferred from macro names. The original deep fixture,
source checkpoint and failure remain preserved. No parser limit was changed.

## Measured ownership/work

Cold layouts: Session 16, Memo 80, Definition **136** (128-byte prior record plus
one optional Rc pointer), SourceCore **176** unchanged; the ordinary Package
256-byte assertion passed. No hot Body/Code/Frame/Value/Type record gains a field.
Unannotated definitions allocate no HeaderData; ordinary/no-source or physical
queries allocate no memo. An actual unannotated macro frame retains a checked
Plain negative row.

List widths 8/32/128 have first-query work 204/682/2630. Repeated uses at
8/32/128 each cost 20 charged units per use, independent of that list width,
with 296 bytes of retained memo payload. Repeated empty attributes have first
work 237/861/3357 and the same 20 units per warm use. Metadata-only widths
8/32/128 cost 1536/6656/28672 for eight repeated sweeps and retain
1088/4112/16208 bytes of memo payload.

For actual origin depths 8/32/128, 32 repeats visit **256/1024/4096 links** and
288/1056/4128 origin lookups, charging 2816/11072/50048 units. Header/frame memo
rows remain exactly one/one. Header/frame authentication is memoized; origin
links are traversed per use. This does not establish universal constant-time
queries, collision-resistant maps, a whole-heap bound or linear whole-compilation
cost. Integration still needs performance qualification, including ordinary
macro controls, list/owner width and retained-origin depth times uses.

## Integration boundary and next consumers

Initial source fixtures/docs are `cd0ff22f`, `5d8d7d4f`; complete-list proposal
and additions are `dba51e4c`, `d7e4d927`, `bb6a88d5`, `94c691f3`. Compiler/test
commits are `a9da27ea`, `cb7bfeb4`, `45c9a757`, `222b637a`. Tooling-only
`b2275423` is the already-deployed `4f34c6c7` patch and is not a compiler delta.
The accepted spelling-repair base is `a7201d48` (same patch as be1c769b); the
compiler/test net delta is a7201d48..222b637a. Keep the earlier repair and all
historical failure receipts distinct.

Root must retain its integration-specific fourth `Paths::invocation` call in
panic_dispatch and its actual definition-scope publication. This worker base
still contains ancestry.append at that unchanged site. Preserve the root's
Attributes witness regression opponent and measured SourceCore assertions when
merging; the Attributes identity itself is present in this source.

This branch also contains older independently unqualified fields from b937.
They are not part of the allowance net delta, have no production dependency
from it, and are not qualified by the 31 allowance hypotheses. Their 24-field
source/native corpus remains required before LIB002 acceptance. The separate
anonymous selector checkpoint379 and LIB004 Copy readiness ownership remain
unchanged and outside this slice.

Real unstable-use integration remains pending. Existing
resolve/stability/{attributes,inventory,owners} implements local declaration
inventory and reachability, not use authorization. Inventory::complete rejects
dependency stability; sema/crates/{export,authored_export} reject its metadata.
The first foreign API consumer must follow actual authenticated policy transport
and resolved References/imports, check the callee's defining crate and the exact
use token, then consider caller feature requests or a session query for that
actual Policy feature. Missing/foreign/stale metadata and callable/const/trait
permissions remain independent rejections. Same-crate regular API uses are
already allowed and do not need a fabricated grant check.

The separate language-feature consumer bindings/admission::require currently
receives Package during loading and after expansion (NeverType, expression
attributes, const traits, auto traits, negative impls). A future integration must
retain early physical-source checks and use an explicit final Report/session
borrow with the actual feature token's provenance. It cannot replace package
feature requests with a blanket owner allowance. No such new grant is claimed
or implemented here. Combined integration release/text/source/native and
performance checks, followed by a fresh merged unchanged-core observation,
remain integrator work.
