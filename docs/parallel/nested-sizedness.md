# Placeholder sizedness in negative coherence

This is the Lane B item of 2026-11-09, on origin/main `7e8c5cbbf`. It fixes
the residual that the [premise-sides batch](negative-premise-sides.md)
recorded (its section 6). The repro was `nested-premise-side`, which rustc
1.98 rejects with E0119 and this compiler accepted with the blanket first:

```rust
pub trait Seal {} impl<T> !Seal for &T {} pub struct Wrap<X: ?Sized>(X);
#[rustc_strict_coherence] pub trait Holds<X: ?Sized> {}
impl<'a, V, S> Holds<&'a Wrap<V>> for S where &'a Wrap<V>: Seal {}
pub struct Local; impl<U: ?Sized> Holds<&Wrap<U>> for Local {}
```

## 1. Rule

rustc 1.98 negates impl2's predicates under `param_env(impl1)`, in both
orders, with each unified header variable a placeholder, and resolves
regions with no WF types. A placeholder is Sized only through that
`param_env`. Any other type is Sized by `sized_conditions`:

| Kind | Sized when |
| --- | --- |
| tuple | its last element is Sized (the empty tuple always) |
| array, array term | always |
| nominal | its sizedness constraint holds |
| pattern | its base is Sized |
| slice, `str`, trait object, foreign type | never |
| rigid parameter | only through the `param_env` |

A written premise of the other header, such as `where Wrap<U>: Sized`, holds
for its exact type.

## 2. What this compiler did

`freeze` makes the first header's binding the representative of each
unified parameter. The interned Sized and well-formedness properties of a
type with parameters are computed from declarations when it is interned
(`types/mod.rs:640-702`, `types/sizedness.rs:184`), so they belong to the
representative. The negative query read them in four places:

- its builtin proof, `Proof::for_unvalidated_implementations(None)`, has no
  shapes, so `normalized_shape` answered from the interned `is_sized`;
- for a projected-size type, `prepare_shapes` built shapes under an empty
  environment, whose parameter arm fails with "shape parameter is outside
  its environment";
- `pay_negative` paid a non-parameter Sized debt with `Proof::new(None)`;
- the nominal size query's parameter arm (`sizes/query.rs:236`) read the
  declaration.

So `Wrap<V>: Sized` held by the blanket's Sized `V` with the blanket first,
and failed with the client first. The before-runs (base CLI `56c5fad9…`)
found seven order-dependent twin pairs and two pairs that rustc accepts but
this compiler rejected in both orders. All four projected-size twins failed
with the shape-parameter error.

## 3. Change

- **Types (`types/parameters/shape.rs`, `shape/size_demands.rs`).**
  `Placeholders` holds the other header's Sized set. `Placeholders::new`
  orders and deduplicates it by identity itself, so no lookup depends on a
  caller's order. Its work is returned to the caller's budget: one comparison
  per premise for an ordered set, which is shared as it is, plus
  `n (ilog2(n) + 1)` for a sort. A lookup is a binary search charged
  `ilog2(n) + 1`. `Shapes::with_placeholders` walks only the children
  `sized_conditions` names and decides sizedness by that table, in a match
  that lists every kind, so a new kind cannot be Sized by default. A rigid
  parameter is Sized only as a member, and any structurally unsized type is
  Sized as a written member. A projection or an opaque type is Sized by its
  own declared bounds. rustc negates under non-body analysis, where an opaque
  is a rigid alias whose item bounds prove it Sized unless they relax it.
  Every opaque declaration here is Sized, because opaque result bounds cannot
  be relaxed. Every type is well-formed: rustc
  checks no WF here, and each header was checked under its own declarations.
  No parameter declaration and no interned property of a type with
  parameters is read. A closed type keeps its interned answer, which no
  representative can change. The normalized sub-walker (one
  `normalized_query` for both of its sites) and the clause-term walker
  inherit the set. `types` names no sema type.
- **Nominals (`sema/nominals.rs`, `nominals/sizes/query.rs`).**
  `Proof::assume_placeholders` installs the placeholder shapes on a fresh
  premise-less proof, and its clause-term shapes inherit them. A placeholder
  proof keeps its shapes even when a walk fails, so the interned fallback of
  `normalized_shape` is unreachable for it. A fenced walk is cancelled
  (`Shapes::refuse_size`), never kept suspended, so a later query on the same
  proof starts fresh. `sizes::Query::with_placeholders` follows the same
  table, also in a match that lists every kind.
- **Traits.** `NegativeSizing` holds the `Placeholders`, so (f) and the
  walks read one set. `may_overlap_with` charges its construction and
  installs the placeholder builtin proof, `storage_condition` passes the set
  to its size query, and `pay_negative` charges its own construction and
  pays a non-parameter Sized debt with a placeholder proof.

### Fence (decision 1, C3)

A projected-size nominal's size is answered by a nested trait proof that has
no header premises. Under placeholders, the builtin proof's walk and the size
query reject it with "projected-size nominal in a negative builtins proof
requires header premises in its nested size proof", unless it is a written
member. This is a known rejects-valid for `nested-projected-size`, which rustc
accepts. B, threading the set into the nested trait proof, is a separate item.
A projection the size query meets still descends into the same negative
query, which has the set.

[Item B](negative-nested-proofs.md) retires this fence at its checkpoint 3b:
the proof's trait State decides the size under the same premises, and
`nested-projected-size` and its swapped twin are accepted, as rustc accepts
them.

### Premise-less behaviour of each shared change

| File | Without placeholders |
| --- | --- |
| `shape.rs` | `normalized_query` keeps the two sub-walker constructions; `premise_sized` is `assumes_sized`; `sizedness_argument` keeps the old error text |
| `size_demands.rs` | `query` calls the same sub-walker; only a placeholder proof calls `refuse_size` |
| `nominals.rs` | `shape_properties` drops the shapes on error, as before; `for_clause_terms` gets `None` |
| `sizes/query.rs` | `placeholder_begin` and `fence` return at once |
| `sizing_fallback.rs` | (f) computes the same lookup and work |
| `sized_obligations.rs`, `coherence.rs`, `inputs.rs` | change only queries with premises |

`source_tail_condition`'s size query is never advanced (it holds a tail
goal), so it takes no set. A comment at the call site says that a change
that advances it must thread the set. `has_unsized`, the single-header comparisons and
stable-mode `may_overlap` pass no premises.

## 4. Controls

`pa31/tests/spec/negative-premise-sides/` gains twenty-six twins; its README
has the table and `outcomes.json` the records. With the existing pair:

- 7 rejected twins become accepted, as rustc accepts them: client-side,
  Copy client-side, written premise, tuple head and array twins;
- 4 accepted twins become rejected, as rustc rejects them: the residual,
  `nested-deep`, `nested-tuple-tail` and `nested-metasized`;
- the four projected-size twins change from the shape-parameter error to the
  fence;
- the other twin of each order-dependent pair, `nested-metasized-held`,
  `nested-wf-control`, the probe pair and the 19 other earlier cases are
  unchanged.

No accepted twin that rustc accepts is rejected, and no twin that rustc
rejects is accepted.

## 5. Amendments

Both are dated 2026-11-09 and kept beside their declarations:

1. Before any oracle or RustGM run, decision 1 changed from A to C, and
   `nested-projected-premise-side` was added.
2. After the oracle (all 26 as declared) and the before-runs (22 of 26),
   before any code change: the four projected-size twins, declared accepted
   before, were the shape-parameter error in both orders. Decision 1 became
   C3, the fence in both orders.

The declarations are `predictions6.json` (`2b261a8c…`), with its predecessors
`04ccc20c…` and `b731d0f9…`.

## 6. Invariants

- `coherence_premise_sides.rs`: 47 cases and 17 swapped pairs. Every pair has
  an equal diagnostic text, with no exemption, and every accepted case is
  disjoint in both orders. `nested-projected-size` is named as the known
  rejects-valid.
- `placeholder_sizing.rs`:
  - the `sized_conditions` table for a parameter, `Wrap<p>`,
    `Wrap<Wrap<p>>`, a tuple head and tail, an array, a slice and `()`, with
    the empty set, `{p}` and the written member `{Wrap<p>}`. Answers and
    work are equal for a Sized and a `?Sized` representative, whose interned
    properties differ;
  - the sub-walkers and the placeholder proof inherit the set;
  - a projected-size nominal is the fence in the proof and the size query;
  - a fenced proof cancels its walk: it fences the root again for the same
    work, then answers a closed type and `Wrap<p>` as a fresh placeholder
    proof does, for the same work;
  - `Placeholders::new` orders and deduplicates a reversed set with
    duplicates, with the same lookups and the sort's work;
  - the empty-set control: closed types keep their answers and work, rigid
    parameters are never Sized, a nominal chain costs exactly one lookup per
    node, and no type costs more than twice its premise-less walk;
  - (f) and the walk give the same answer for the same work, for every rigid
    parameter of every analyzing packet fixture;
  - work is linear in depth: a parameter `d` nominals deep against `n`
    premises, for `n` up to 4096, costs at most `3 (d + 1) (ilog2(n) + 1)`
    steps;
  - a header that fails well-formedness is rejected before coherence, in
    both orders.
- Pins unchanged: `sharing_sizing` Query 696, Node 160, Frame 1312.

## 7. Evidence

- **Golden table (a).** A test-only hook (never committed) printed rows for
  the 60 fixtures of `negative-premise-sides` and `negative-metasized`. For
  every type of each analyzing fixture, it printed the premise-less walk
  under the fixture's parameters and under no parameters, the premise-less
  builtin proof and the size query. It covers only the premise-less paths of
  the fixtures that analyze: 25 on the base, 28 on the candidate. Base
  `table.txt` sha256 is `7c140bd0…`. The 21 fixtures that analyze on both
  trees give byte-identical rows (2058 rows). The other 4 on the base and 7
  on the candidate are the declared flips.
- **Flip survey.** The base and candidate CLIs (`de490950…`) give
  byte-identical output over the same 336 fixtures as the premise-sides
  batch. A never-committed probe build (`b0478f2e…`) marked three points:
  placeholder proof installation, the placeholder walk's decision for a type
  with parameters or projections, and the start of a placeholder size query.
  Its survey output equals the candidate's.
  - In the survey, 19 fixtures install a placeholder proof (17
    `negative-coherence`, 2 `fn-pointer-coherence`). None reaches the walk's
    decision or a size query.
  - In the packets, 59 of 60 install one and 34 reach the walk's decision.
    The four projected twins stop at the fence before it.
  - No fixture starts a placeholder size query, so only the unit tests cover
    that path. B's notice must include at least one packet fixture that
    reaches it.

  A closed type under placeholders is answered from its interned property
  before the decision. That is unchanged by construction, and the empty-set
  control checks it.
- **Census lookahead.** Analysis-only, over census 19's manifest. The base
  and candidate `census.tsv` are byte-identical (sha256 `6b88904a…`), so the
  item moves no census record.
- **Review re-check.** After the review changes (premise ordering, the
  exhaustive matches and the cancelled fence), the final CLI (`47f65f80…`)
  was compared with `de490950…`:
  - the 60 twins give byte-identical outcomes;
  - the 19 survey fixtures that install a placeholder proof, plus the 60
    packet fixtures, give byte-identical output. The review changes run
    only under an installed placeholder proof, so the other 317 survey
    fixtures cannot reach them;
  - the census lookahead's `census.tsv` is again `6b88904a…`.

## 8. Open items

- B: thread the header premises into the nested trait proof, for projected
  sizes and for normalization inside the builtin proof. Done by
  [item B](negative-nested-proofs.md).
- `probe-trait-premise`: a rejects-valid probe pair, which rustc accepts. A
  negative impl's trait condition is not assumed from the other header's
  trait premises.
