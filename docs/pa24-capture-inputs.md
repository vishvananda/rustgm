# Typed capture input checkpoint

PA24 remains planned. This checkpoint connects resolved free bindings to the
existing capture graph and retains the facts required by ordinary body,
ownership, lifetime and lowering consumers. Captured closure execution is not
yet qualified here; its consumer work is separate.

A lazy lexical index supplies imported inference inputs. Signature, complete
closure identity and free binding types cross the body boundary together, using
one lifetime opening. Actual typed expression uses determine capture modes;
lexical occurrence alone does not grant a borrow or move.

The iterative use collector caches source places and interns shared projection
prefixes. It retains direct access mappings and separate wildcard initialization
mentions. Value use consults the existing Copy proof; numeric inference domains
are known Copy, while an undetermined general input remains an explicit
boundary. Move construction changes stored capture mode independently of the
closure's Fn/FnMut/FnOnce call permission. Shared-reference truncation and unique
immutable capture use the existing graph rules. Graph finalization retains a
direct covering-capture index, avoiding repeated ancestor searches.

The environment retains path types in both the child proof context and the
immediate parent's formation context. Selected field types, binding mutability
and whole Self are retained explicitly. The hidden execution receiver has a
separate signature and lifetime count from the public callable signature. Its
body and call consumers must still prove ordinary permissions; these facts are
not a bypass for ownership or lifetime checks.

Seven unchanged, independently accepted PA24 inputs check shared, mutable,
move-but-Fn, consuming, precise field and unique immutable capture facts. A
wildcard control verifies that a mention does not create an environment field.
Evidence is in `obj/parallel/capture-inputs/collector-v2/` and `collector-v4/`:
37 focused source test functions pass; after replacing compile-time fixture
inclusion with ordinary file loading, the three capture tests pass again.
Five native regression groups covering existing closure/lifetime, inline
constant and raw-read behavior pass. Formatting, strict release all-target and
all-feature Clippy, layout and the file audit pass (1,318 files, zero findings).
The initial include-provenance audit finding and formatting failure are retained
with their failed receipts. Successful scoped cleanup reclaimed 0.31 GiB.

Remaining collector boundaries include adjusted place uses, method receivers,
compound patterns, indexing, packed/union/raw-pointer precision, nested capture
propagation and unresolved composite Copy inputs. Generic captured execution,
receiver adapters and complete source/native capture qualification remain
separate work. No text oracle or assignment activation changed.
