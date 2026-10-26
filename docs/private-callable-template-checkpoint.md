# Private callable-template checkpoint

Source `7a258d6c1d22e5a221887765f31f5ac5579bd2ac` prepares genuine
reserved named callable templates without publishing headers, environments,
roles, names or executable bodies. Public Provided import and execution remain
closed. This is a prerequisite for the original library artifact, not that
artifact's acceptance.

## Owner and registration

The private entry consumes and returns the existing owned session:

```text
Session::callable_template(Box<Self>, DeclarationRef, &mut ReservationWork)
    -> Result<(Box<Self>, Demand<()>), Error>
```

The session retains the actual moved resolver Report, Declarations/type Store,
immutable inventory and reserved Catalog. It reuses the persistent RecipeCursor
and genuine SignaturePair. No TypeId, Templates, Graph, Header or Environment
escapes this interface. Interned identities remain monotonic; failure drops the
unpublished session and preserves attempted-work and peak accounting.

Cold preparation collects only the selected callable's actual ancestry and full
type-formal vector, including unused parameters and inherited Self. The ordinary
importer and this private owner share one existing registration kernel:
`insert_monomorphic` for a genuinely empty complete vector, otherwise
`insert_scoped`. Ordinary eager ordering, preflight and symbol production stay
unchanged. No dummy generic or empty placeholder template is introduced.

Every warm request checks inventory, Catalog, arena, actual declaration/body row,
signature and immutable template seals. A constant-time formal-scope seal retains
the actual scope index and full ParameterScope identity. Removing or replacing
the selected scope rejects even while its Ready record remains present. Parent
ancestry and full vectors are checked at cold registration; warm requests do not
walk them again.

## Work and storage

The private cold owner has dense scope, readiness and occurrence tables, charged
once from immutable metadata. Selected registration charges actual parent/formal
work. The cost-only Templates accessor bounds real collection-capacity growth and
temporary owner validation before insertion. Actual retained growth is reconciled
on success; failed preflights keep their charges. Regional pointer occurrence
preflight follows the selected shared reference frontier, without a whole Store
or catalog envelope or unrelated CFG traversal.

The retained Templates representation stores each callable's complete formal
vector. Shared recipe nodes do not make those vectors shared storage. On the
qualified x86-64 build, Ready is 256 bytes, Prepared is 152 bytes and SignaturePair
is 28 bytes; collection capacity is additional. No hot Body, Contract or Store
field was added.

Actual selected controls report:

| Nested Function depth | Expanded recipes | Edges | Work for 1,000 warm requests | Private owner retained bytes |
| --- | --- | --- | --- | --- |
| 1 | 3 | 4 | 30,000 | 2,464 |
| 4 | 9 | 16 | 30,000 | 2,536 |
| 8 | 17 | 32 | 30,000 | 2,632 |

Unused-formal widths 1, 8 and 32 retain 2,424, 2,608 and 3,280 bytes in the private
owner. These are deterministic capacity charges, not allocator or RSS samples.
Warm requests preserve memo counters, interner extent and retained capacity.
Unrelated-original counts 1, 8 and 32 preserve equal warm work and no growth.

The actual full-family FunctionItem body dependency remains Blocked. One new
recipe is expanded, then 1,000 template requests and a final signature-pair check
record 7,008 work units without further expansion or allocation. The target pair
and template never become Ready. Named function lifetimes retain their earlier
Catalog denial: the source-free declaration installer does not yet admit the
actual Lifetime declaration kind. This checkpoint does not bypass that boundary.

## Focused qualification

[CI 40219671209](https://github.com/vishvananda/rustgm/actions/runs/40219671209)
passed metadata, formatting, global architecture audit, strict private Clippy and
the unoptimized harness build. Four of six controls passed: the original complete
family and unused/monomorphic distinction, changed provenance/formal-scope
opponents, ordinary Functions public-codec/fresh-client proof and frozen Catalog
denial. Two setup failures were retained: a scalar literal outside the transport
profile and the earlier named-lifetime Catalog boundary.

[CI 40220578997](https://github.com/vishvananda/rustgm/actions/runs/40220578997)
passed the corrected two controls in 0.165 seconds. It authenticated the prior
four passes by all 2,594 source fingerprints and actual Rust function/nonfunction
syntax nodes, allowing only the two corrected tests and their new Blocked helper.
Changed-file formatting, global duplicate/architecture audit, private Clippy and
the harness build passed again. The continuation took 157.96 seconds, plus the
mandatory workflow CLI build's 39.14 seconds. The workflow also passed 215 harness
and seven audit-cache tests. The preceding helper-interpreter failure reached no
checks or tests and remains separate evidence.

The qualified source fingerprint is
`0edad093a1b376c8eafd0589f52eef7cf0985a4c1e963111a0d8a8544759ef19`;
the continuation receipt SHA-256 is
`815731cb534d8c63b5af5e3ab260e843ef6fb9dc6f675196a889cbac5227292b`.
Local evidence is under
`obj/parallel/pa29-source-client-staging/lib011/private-callable-templates-v1/`.
No local Cargo, release, native or full-private sweep was run for this checkpoint.

## Remaining authority

The original four-function/three-body/nine-role family remains intact. In
particular, clone_from's ordinary declaration and actual const-definition body
context remain distinct; ordinary u8 Clone gains no const-callability permission.
Template readiness pays no well-formedness, trait, region, loan, original-body or
fresh substituted-instance obligation. Declaration versus body environments,
normalization/numbering and alias ownership, named lifetimes, attribute/UB policy
transport and actual default execution remain separate requirements. No HRTB,
type-alias, deep-ready-128 or new FunctionItem/ApplicationSignature admission is
claimed.
