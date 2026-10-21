# PA30 symbol identity checkpoint

`support::linkage` defines `Naming::{Rust, Exact(SymbolNameId)}` and
`Linkage::{Import, Export, Internal}`. `SymbolNames` owns exact UTF-8 spellings
in the existing name arena, without source identifier normalization. Names must
be nonempty and contain no NUL. Raw prefixes, apostrophes and canonically
different Unicode bytes remain distinct. A `SymbolNameId` belongs only to its
owning pool; metadata must transport the bytes and intern them in the receiving
pool, never serialize or reuse the local ID.

`lower::linkage::select` takes the canonical callable instance or static
identity and a lazy Rust naming provider. Exact names bypass that provider and
its future crate identity encoding. The pinned LowIR grammar supports only
ASCII letters, digits, underscore, dot and dollar. Unsupported exact spellings
produce an explicit lowering error while their metadata bytes remain intact.
Ordinary callable demand uses this selector and retains its existing names.
Internal generated closure, drop, vtable and constant-storage labels remain
owned by their existing lowering routines.

Four focused tests cover exact interning, metadata reinterning, Unicode/raw
prefix distinction, lazy callback behavior, canonical callable identity and the
backend vocabulary boundary. The existing independent callable convention
controls and five native cases check the ordinary function path. Foreign
registries, declaration-only function/static lowering, naming attributes and
crate-qualified Rust naming are separate followups. PA30 remains planned and
PA26's remaining hook families remain listed in its checkpoint documents.

Strict all-target/all-feature Clippy and all seven focused tests pass. The syntax
audit reports 1,210 files and zero findings; assignment discovery and four earlier
callable byte views agree. Evidence remains in `obj/parallel/pa30/naming/` and
resource receipts `c99148647266440da73ec089e6e2833d` (focused/native) and
`0e0883f8ad75492d87429b25300416a3` (audit and earlier views).
