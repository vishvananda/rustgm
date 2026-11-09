# Negative coherence under the other header's premises

Twenty-one no_core source-core libraries, checked against rustc 1.98. Each
pair can be disjoint only through a negative impl, and they ask the same
question: which header's premises may pay the negated clause's debts?

rustc's `impl_intersection_has_negative_obligation` runs in both orders.
Each order assumes `param_env(impl1)`, after plugging the unified header
variables with placeholders, and tries to prove the negation of each of
impl2's elaborated predicates. Then `resolve_regions` runs with no WF types,
and a region-region bound between two inference regions is never added to
the free-region relation. So:

- a negated clause's own header pays none of its debts;
- a rigid intersection parameter is Sized or `MetaSized` only through the
  other header's premises, whichever header's declaration `freeze` made the
  representative;
- a lifetime obligation holds only through the other header's written
  type-outlives premises, never through formed types or region-region bounds.

RustGM had paid Sized debts from both headers' premises. A rigid parameter's
sizedness came from the declaration of the first header's binding. `prove`
assumed both headers' regions, outlives and formed types, and an unpaid
region obligation was a hard error ("required lifetime does not outlive its
target"). That made the outcome depend on which header came first.

| Twin | Shape | rustc 1.98.0 | Before | After |
| --- | --- | --- | --- | --- |
| `sized-premise-side-swapped` | blanket's own Sized `V`, client first | E0119 | overlap | overlap |
| `sized-client-side` | client's Sized `U`, blanket's `?Sized` `V` | accepted | overlap | accepted |
| `sized-client-side-swapped` | the same, client first | accepted | accepted | accepted |
| `copy-premise-side` | blanket's own `V: Copy` | E0119 | overlap | overlap |
| `copy-client-side` | client's `U: ?Sized + Copy` | accepted | overlap | accepted |
| `premise-side-swapped` | blanket's own `V: ?Sized`, client first | E0119 | overlap | overlap |
| `metasized-sized-representative` | `?Sized` negative impl, blanket's Sized `V` first | accepted | accepted | accepted |
| `metasized-sized-representative-swapped` | the same, client first | accepted | accepted | accepted |
| `region-client-side` | client's `'q: 'p` | E0119 | accepted | overlap |
| `region-premise-side` | blanket's own `'y: 'x` | E0119 | accepted | overlap |
| `region-absent` | no `'q: 'p` | E0119 | region | overlap |
| `region-head-unified` | client's `'q: 'p`, blanket's own clause | E0119 | region | overlap |
| `region-implied` | `&'p &'q u8`, implied only by WF | E0119 | accepted | overlap |
| `region-strict-control` | no negative impl | E0119 | overlap | overlap |
| `strict-unconditional` | negative impl with no obligation | accepted | accepted | accepted |
| `strict-static-unheld` | needs `'q == 'static` | E0119 | region | overlap |
| `type-outlives-held` | client's written `U: 'p` | accepted | accepted | accepted |
| `type-outlives` | `U: 'p` implied only by WF | E0119 | accepted | overlap |
| `type-outlives-premise-side` | blanket's own `V: 'y` | E0119 | accepted | overlap |
| `nested-premise-side` | blanket's Sized `V` under `Wrap<V>` | E0119 | accepted | accepted |
| `nested-premise-side-swapped` | the same, client first | E0119 | overlap | overlap |

"Overlap" is "overlapping trait implementation headers require coherence
proof", and "region" is "required lifetime does not outlive its target". The
region, strict and type-outlives twins put `#[rustc_strict_coherence]` on
`Holds`. Their goals name only foreign types behind references and tuples,
so without it knowability alone makes each pair disjoint in rustc.
`sized-premise-side` and `premise-side` are in `spec/negative-metasized/`;
`sized-premise-side` is amended there from accepted to overlap.

Every outcome but one is rustc's. The recorded residual is
`nested-premise-side`, which is accepted while rustc rejects it with E0119.
`Wrap<V>: Sized` is not a parameter goal, so the shape walk still decides it,
and the walk reads the declaration of the representative that `freeze` chose
(`types/parameters/shape.rs:285-290`). With the client first,
`nested-premise-side-swapped` is rejected as rustc rejects it. Fixing this
means threading the query's premises into that shared types facility, which
is a separate design.

Outcomes were declared before each twin's frozen rustc 1.98.0 oracle and
before any RustGM run of it. `outcomes.json` records both, with these
amendments:

- **Strict correction.** The first five region twins were rewritten with
  `#[rustc_strict_coherence]` before any RustGM run. Without it, knowability
  had already made them disjoint in rustc.
- **Wrong rustc predictions.** `region-client-side` and `region-head-unified`
  were predicted accepted, and rustc rejects them: it never assumes a
  region-region premise between two inference regions.
- **Wrong before predictions.** `copy-premise-side`, `sized-client-side` and
  `sized-premise-side-swapped` were declared accepted before the base runs,
  and each was rejected with the overlap text, because of the
  representative's declaration. On main, `sized-client-side` is a
  rejects-valid.
- **Probe-derived controls.** rustc's outcomes for `strict-unconditional`,
  `strict-static-unheld`, `type-outlives-held` and `type-outlives` were
  observed first as probes. Their RustGM outcomes were declared before any
  RustGM run.

Each file is one crate feature line, then the same pinned prefix as
`spec/negative-metasized/`, then the case body. They keep Rust's
MIT/Apache-2.0 attribution; see the root NOTICE and `third_party/sources.json`.

`dev/tests/invariants/coherence_premise_sides.rs` checks every case. Its
symmetry check covers the whole packet:

- every `X-swapped` twin has `X`'s outcome;
- every accepted case is disjoint with its `Holds` pair passed to
  `overlap::possible` in both orders.

The residual pair is exempt by name. The invariant's `RESIDUAL` constant is
`nested-premise-side`, and its comment cites the residual's text (`residual`
in `outcomes.json`): `Wrap<V>: Sized` is still decided by the shape walk,
which reads the declaration of the representative that `freeze` chose. The exempt pair's two orders must still differ. `nested-premise-side`
is disjoint with the blanket first and not with the client first, and the
twins are accepted and overlap. So a fix must update the record.
See `docs/parallel/negative-premise-sides.md`.
