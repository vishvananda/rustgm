# Parsed object fields in source derives

The pinned std consumer derives Copy and Clone for `EHContext<'a>`. Its two
borrowed fields have the type `&'a dyn Fn() -> *const u8`. The fixture preserves
the whole original item byte-for-byte and checks a separate copy and clone use.
It does not invoke the object or reconstruct the original personality body.

The packet also declares an explicit nonstatic object lifetime and a
higher-ranked callable input/output pair. It rejects owned and mutable
trait-object fields when deriving Copy and Clone, a dyn-incompatible
principal, and an attempted escape to a static lifetime. The manifest binds
all the source bytes, including the independent foundation, before the oracle
runs. No native, complete std, or assignment release claim is made.

The plain foundation and `foundation-dyn.rs` gate the independent metadata
cases. The latter exercises Copy and Clone on the actual borrowed callable
field before the semantic cases run. The manifest lists it as setup-only,
and the candidate checks require its expansion, named copy/clone body and
complete source proof before reading the unchanged main case table. The
source proof uses the prepared pipeline so the lifetime-generic bodies are
checked as templates.

The dyn setup also coerces a shared borrow of a closure to exercise the real
DynMetadata role before the seven main cases.
