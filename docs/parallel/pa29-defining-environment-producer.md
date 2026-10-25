# Shared defining body environment producer

Source body checking now uses `function_context::assumption_environment` to
compose its defining environment. The actual declaration Arc stays unchanged.
The source checker obtains `default_self_clause` and translates defining
clauses through its existing cached universal substitution. The shared producer
extends those clauses, derives builtin region equalities with the existing
Constraints/Queries, then attaches the body's actual ordered alias ledger.
Only the completed result is published in body_environments.

The ordinary path with no defining clauses or regional builtins returns the
same declaration Arc without allocating or querying imported declaration input.
The source adapter borrows complete existing parameters, lifetime bounds and
clauses. The lower-level Scope/Context API also accepts checked staged import
facts before declaration tables are installed; it supplies construction only,
with no implementation or execution permission. Import callers must authenticate
owner/signature/environment/arena, reserve their full envelope and supply the
actual checked alias ledger before using it.

Formatting, file/architecture audit and all-target/all-feature strict Clippy
passed in `lib008/env-producer-static-v3` (receipt SHA-256
`e4b11ee1ee56a124047246c207adf98f4d4bf8aec66a42977dce161cae244550`).
The first two gates retain their filename/style failures as separate evidence.
The next gate passed 22 exact focused cases, CLI flag checks and fresh API/CLI
source artifact native consumers, each printing exactly `7\n`; its receipt is
`lib008/env-producer-focused-v1`, SHA-256
`80fce3c3136027a4a6d058d7d63a0d5cb1728959124f6d2a0c575cffe6782525`.
The full original family fixture stayed byte-identical, including ordinary u8
Clone's const-callability limit. Required actual source controls cover local
default Self assumptions versus declaration input, full aliases, rejected
incomplete alias scope, the unchanged ordinary Arc, and mutual-versus-one-way
builtin lifetime equivalence.

The 22 tests remain focused development evidence. The same private binary
SHA-256 `d7c83e07e92018cd1f6112d875dcda76f7c474b17925940892bb513a10fd72ca`
then executed its complete captured nonignored inventory exactly once:
**1,561 passed, zero failed**, with two ignored cases enumerated separately and
all 22 required names checked. This used the existing binary without rebuilding
or repeating static/native checks. The full inventory receipt is
`lib008/env-producer-full-v1`, SHA-256
`585b36c8c8c8baa9d931cf7f1bc2d735100c125526822e48bd106f6e7236899d`.
Source, tools, fixtures and executable hashes remained unchanged. This closes
functional qualification of the extraction; no new performance result is
claimed for this source change.

No source profile, codec, numbering domain or imported default is activated by
this change. Approved review-only Provided/numbering designs remain separate;
actual declaration Scope.environment must stay distinct from callable body
header.environment. Raw Contract/Required identity, original region/loan and
fresh instance proof remain required before imported activation. The
publishing-only attribute boundary is a separate focused follow-up; no old
metadata or map omission establishes KnownAbsent.
