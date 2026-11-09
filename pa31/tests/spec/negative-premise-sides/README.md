# Negative coherence under the other header's premises

Forty-seven no_core source-core libraries, checked against rustc 1.98. Each
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
| `nested-premise-side` | blanket's Sized `V` under `Wrap<V>` | E0119 | accepted | overlap |
| `nested-premise-side-swapped` | the same, client first | E0119 | overlap | overlap |

The nested-sizedness item (2026-11-09, on `7e8c5cbbf`) adds twenty-six twins.
Each is `X` with its blanket first and `X-swapped` with the client first.
"Before" gives `X` / `X-swapped` with the `7e8c5cbbf` CLI; "fence" is the
named fence below.

| Twin | Shape | rustc 1.98.0 | Before | After |
| --- | --- | --- | --- | --- |
| `nested-client-side` | client's Sized `U`, blanket's `?Sized` `V`, under `Wrap` | accepted | overlap / accepted | accepted |
| `nested-copy-client-side` | client's `U: ?Sized + Copy` | accepted | overlap / overlap | accepted |
| `nested-written-premise` | client's `U: ?Sized` with `where Wrap<U>: Sized` | accepted | overlap / overlap | accepted |
| `nested-deep` | `Wrap<Wrap<V>>`, client `U: ?Sized` | E0119 | accepted / overlap | overlap |
| `nested-tuple-tail` | `(u8, V)` | E0119 | accepted / overlap | overlap |
| `nested-tuple-head` | where `&'a (V, u8): Seal` over `&'a V` | accepted | accepted / overlap | accepted |
| `nested-array` | where `&'a [V; 1]: Seal` over `&'a V` | accepted | accepted / overlap | accepted |
| `nested-metasized` | `?Sized` negative impl, `WrapP<X: PointeeSized>`, client `U: PointeeSized` | E0119 | accepted / overlap | overlap |
| `nested-metasized-held` | the same with client `U: ?Sized` | accepted | accepted / accepted | accepted |
| `nested-wf-control` | `(V, u8)` with Sized on both sides | accepted | accepted / accepted | accepted |
| `nested-projected-size` | `struct P<T: ?Sized + Tr>(<T as Tr>::Out)`, Sized on both sides | accepted | error / error | fence |
| `nested-projected-premise-side` | the same with client `U: ?Sized` | E0119 | error / error | fence |
| `probe-trait-premise` | `impl<T: Tr> !Seal for &T`, client `U: Tr` (probe) | accepted | overlap / overlap | overlap |

"Overlap" is "overlapping trait implementation headers require coherence
proof", and "region" is "required lifetime does not outlive its target". The
region, strict and type-outlives twins put `#[rustc_strict_coherence]` on
`Holds`. Their goals name only foreign types behind references and tuples,
so without it knowability alone makes each pair disjoint in rustc.
`sized-premise-side` and `premise-side` are in `spec/negative-metasized/`;
`sized-premise-side` is amended there from accepted to overlap.

"Error" is "shape parameter is outside its environment", and "fence" is
"projected-size nominal in a negative builtins proof requires header premises
in its nested size proof".

The union batch's residual, `nested-premise-side`, is fixed. Under the other
header's Sized premises, the shape walk and the nominal size query follow
rustc 1.98's `sized_conditions` and read no parameter declaration and no
interned property of a type with parameters. So every outcome is rustc's,
except in these cases:

- `nested-projected-size` and its swapped twin are a known rejects-valid.
  A projected-size nominal's size is answered by a nested trait proof that
  has no header premises, so both orders reject it with the fence. On main
  they were the shape-parameter error. Threading the premises into that
  proof (B) is a separate item.
- `nested-projected-premise-side` rejects with the fence in both orders,
  where rustc rejects with E0119.
- `probe-trait-premise` is a probe, rejected in both orders while rustc
  accepts it: a negative impl's trait condition is not assumed from the
  other header's trait premises. It is recorded, with no fix.

Outcomes were declared before each twin's frozen rustc 1.98.0 oracle and
before any RustGM run of it. `outcomes.json` records both, with these
amendments:

- **Projected sizes (nested item, two amendments).** Before any oracle or
  RustGM run, `nested-projected-size` was redeclared accepted after, and
  `nested-projected-premise-side` was added. After the oracle (all
  twenty-six as declared) and the before-runs, before any code change, the
  four projected twins, declared accepted before, were observed with the
  shape-parameter error in both orders. They were redeclared with that
  before-outcome and the fence after.

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

`dev/tests/invariants/coherence_premise_sides.rs` checks every case but the
probe, which only `outcomes.json` records. Its symmetry check covers every
checked pair:

- every `X-swapped` twin has `X`'s diagnostic text;
- every accepted case is disjoint with its `Holds` pair passed to
  `overlap::possible` in both orders.

No pair is exempt: it compares diagnostic texts, and the projected-size
pairs give the same fence in both orders. See
`docs/parallel/negative-premise-sides.md` and
`docs/parallel/nested-sizedness.md`.
