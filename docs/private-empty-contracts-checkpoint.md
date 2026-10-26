# Private empty Contract environments

Compiler `c85ac736377df8a52cf770bada9705bb680dd622`, based on qualified main
`74281adc`, prepares a real neutral body Environment for a genuinely nongeneric
plain function whose raw origin is `Contract`. The owned private Session still
returns only itself and `Demand<()>`. Public Provided import, roles, execution
and proof remain closed.

## Actual construction and identity

`EmptyContract` is a sealed positive witness for the actual function/scope,
original, raw BodyInputs, BodyUnit, original/public binders and complete empty
parameter/clause domain. Both binders must be zero root binders. The actual
root scope has no parent/captures/slots; the complete declaration and regional
input vectors must be empty. Missing rows fail. Nonempty, captured, nonzero,
parent/default and unknown origins do not acquire this witness.

The ordinary `from_contract` wrapper retains its existing guards, slot/clause
mapping order and work/storage charges. Its extracted `ContractParts::ordinary`
and the private sealed empty parts use the same `environments::prepare` producer.
The private path also uses the existing defining `Prepared::begin/finish` kernel.
There is no second Environment, trait or region algorithm.

A separate cold Contract product retains the genuine neutral body Arc and its
origin/body witness. Its declaration Arc is an identity anchor, never the body
product. Every warm hit authenticates actual inventory/Catalog/arena, owner,
body, signatures, template, scope and origin before returning readiness. The
body Arc must differ from the actual declaration Arc. Ordinary Declaration
readiness continues to share that actual declaration Arc.

Source declaration-map absence remains honest: no global completion map is
inserted, reserved or used as a fallback. Raw Contract stays Contract. Original
operations, builtin selectors/requests, obligations, normalizations, numbering
and other proof debts are retained unchanged. No Environment, TypeId, Header,
Graph or permission escapes the private Session. Construction does not prove a
body, loan, operation, const call or substituted instance.

## First-encoding compatibility

Before any adapter edit, actual `build_source_library_profile` plus the first
`crates::encode` captured both old profiles in
[CI 40229239110](https://github.com/vishvananda/rustgm/actions/runs/40229239110).
The source was the original minimal provider plus genuine `plain(x:u8)->u8` and
unused-generic functions, with actual SourceCore identity
`minicore/plain_contract_pair`, default Config and Rust2024. Raw bytes, complete
row views, source and config bindings were retained. This was not a roundtrip
substitute for old bytes.

The post-adapter first encodings, row views, source and config bindings all
match those captured bytes exactly:

| Profile | Bytes | SHA-256 |
| --- | ---: | --- |
| Functions/tag10/v0 | 4,927 | `d0e70e6a27bf868384efdde960bf465bf27969f1419ee165098d38e2465a5ae9` |
| FunctionContracts/tag10/v1 | 5,035 | `da1762df6c39eb9135ffc62985c61f630a2b1f0a17526c9cabb851790cd849e9` |

Both real public artifacts decode into fresh consumers that actually call the
nongeneric plain function. Existing Functions/fresh-client proof also passes.
The four public admission guards, exporter, codec and serialized profiles are
unchanged from main `74281adc`.

## Charged cold storage and warm work

The extra origin seal and two Arcs live in a selected-only 32-byte boxed product.
This keeps every ordinary private Body at its previous 280 bytes. The separately
constructed neutral body Environment retains 112 bytes. The existing Scope's
neutral declaration Environment is distinct and remains separately accounted.
There is no warm allocation or whole-body/whole-catalog reproof.

Actual x86-64 enclosing layouts are Owner80, Scope112, Body280, ContractProduct32,
Templates160, State232, Session10352 and Ready256 bytes. Dense body/scope slots
remain eight bytes. No hot Body, Contract, Store or declaration field changed.

| Unrelated originals | Environment owner bytes | Body capacity/bytes | Scope capacity/bytes | Application capacity/bytes | Logical whole-budget peak |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 1,432 | 6 / 48 | 26 / 208 | 36 / 432 | 166,720 |
| 8 | 1,628 | 13 / 104 | 33 / 264 | 43 / 516 | 243,017 |
| 32 | 2,300 | 37 / 296 | 57 / 456 | 67 / 804 | 484,440 |

Each row allocates exactly one body record/product. The explicit dense metadata
cost grows with the immutable catalog once. Each 1,000 warm Contract demands
costs exactly 79,000 work units for all three sizes, without retained-capacity,
Cursor-counter or interner-extent growth. Existing Declaration demands retain
their 65,000 work units and 1,708/2,072/3,320 owner bytes. Formal widths 1/8/32
retain the preceding checkpoint's exact costs. These are logical capacity/work
measurements, not RSS or allocator claims. Failed Session and actual empty-Contract environment
prepayment attempts preserve work/peak and publish no new product.

## Focused qualification and remaining work

[CI 40230908636](https://github.com/vishvananda/rustgm/actions/runs/40230908636)
passed all six exact groups, formatting, full architecture audit, both production
and harness-mode strict Clippy, and the unoptimized harness build. Tests took
0.214 seconds; the helper took 132.99 seconds and the mandatory production CLI
build 34.79 seconds. Metadata passed in the first adapter gate and is retained
with exact source-delta authentication; its inputs did not change. The workflow
also passed 215 harness and seven audit-cache tests.

Controls include the full original family, real plain/unused/bounded functions,
actual source-map absence, unchanged raw requests, foreign inventory/arena,
missing/replaced scope, actual other-body identity, substituted declaration Arc,
changed origin memo, budget failure, selected/unrelated/warm scaling and ordinary
Arc/codec/fresh-proof sentinels. No compiler expectation was weakened.

Retained failures: the pre-adapter capture first stopped at a redundant debug
format borrow; the first adapter gate stopped at three large-by-value request
warnings and a duplicated warm-control block. Neither reached tests. Borrowed
requests and one shared warm observer corrected those findings without lint or
audit exemptions. No local Cargo, release, broad/private or native sweep ran.

All 2,614 source fingerprints match the passing receipt. Canonical source SHA-256:
`7fa6ef828312c18933f81e58b0e6678f8d32e89694e89d762315c97d99d2330b`.
Receipt SHA-256:
`284fad872735ad09c6145349d4d5a0d96b92abe8e6a9ce495fffbbd753eb18f3`.
Raw baseline, actual post-adapter artifacts, all failures, static receipts and
six selections remain under
`obj/parallel/pa29-source-client-staging/lib011/empty-contracts-v1/`.

Actual inherited/nonrole Holder inputs must next use B's genuine trait producer
and this same owned canonical translator. Nonzero/regional/default/alias domains,
raw Contract/Required identity, region/loan/original proof, fresh substituted
instances and public Provided execution remain separate required capabilities.
Attribute knowledge and UB policies gain no authority from absent old rows.
