# Imported nominal formation packet

`imported-formation-outcomes.json` freezes one provider and 18 clients: their
hashes, edition, target, independent outcomes and diagnostic locations. They
were declared in scratch before the rustc oracle and the baseline probe ran,
and frozen at 2026-11-09T00:30:25Z, after both and before any candidate run.

The provider is `spec/knowable-absence/provider.rs` (unchanged) followed by
authored lines shaped as alloc 1.98.0's `rc.rs`: `pub trait Allocator {}`,
`pub struct Global;` with `impl Allocator for Global {}`,
`pub struct Rc<T: ?Sized, A: Allocator> { pub alloc: A, pub value: T }`,
`pub struct Bounded<T> where T: Copy { pub value: T }` and
`pub struct Nested<A: Allocator> { pub inner: Rc<u8, A> }`. The inherited
lines keep their attribution, and the repository NOTICE applies to them. The
alloc-derived shape follows the Rust project under its MIT/Apache-2.0
licence.

A client forms every imported nominal it names under that nominal's
transported where-clauses, as rustc does. Six clients return 42:

- `rc-global`: `base::Rc<u8, base::Global>`;
- `rc-generic-bound`: `fn hold<A: base::Allocator>(_r: base::Rc<u8, A>)`;
- `rc-client-impl`: a client `impl base::Allocator for Mine {}`;
- `rc-unsized`: `&base::Rc<str, base::Global>`, whose `T: ?Sized` adds no
  Sized requirement;
- `bounded-copy`: `base::Bounded<u8>`;
- `local-positive`: a source-local `LocalRc<A: LocalAlloc>` with a satisfying
  argument.

Twelve clients are rejected with E0277 by rustc. Seven name an imported
nominal whose where-clause fails:

- `rc-not-alloc`, `rc-generic-unbound`, `rc-reference` and `nested-not-alloc`
  in a signature;
- `rc-field` in a struct field;
- `impl-header` in an impl's Self type;
- `bounded-not-copy` through a `where T: Copy` clause.

`slot-unsized` names `&base::Slot<str>`, whose Sized edge was already
enforced. `local-control`, `local-field`, `local-impl-header` and
`local-generic-unbound` are the source-local twins of the signature, field,
impl-header and generic opponents. Each imported opponent is rejected at the
same kind of site as its twin: a signature at its function item, a field at
the field and an impl header at the impl item.
