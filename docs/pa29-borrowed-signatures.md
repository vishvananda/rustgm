# Borrowed function signature metadata

Status: implemented and focused-qualified draft; PA29 remains planned. No
assignment activation or generated reference publication is included.

Separate RustGM libraries can export ordinary Rust functions with scalar or
reference parameters/results. References can be nested, shared or mutable, and
refer to the existing closed scalar family (including unit behind a reference).
Signatures retain explicit late-bound lifetime slots and static references.
Early-bound lifetimes, explicit bounds, type/const parameters, const/async bodies,
nominals, arrays, slices, raw pointers and function-pointer types in exported
signatures remain explicit profile boundaries. A consumer may still reify a
supported imported function into an ordinary local function pointer.

The producer consumes completed canonical `Type::Function` facts and retained
`FunctionRegions`; it does not parse declaration text. Captures, early arguments
and explicit bounds must be absent. Existing original body, lifetime, ownership,
loan, destruction and lowering checks still precede artifact publication. A
function with an early-bound output-only lifetime is valid Rust but outside this
profile; an ordinary static output is supported. Both distinctions have source
and independent object controls.

## Typed transport and wire grammar

`support::crates::Signature` owns a validated signature-local `TypeGraph`, typed
parameter/result `TypeRef` roots, the full `bound_regions` count and unsafe bit.
`Signature::new` retains the existing scalar constructor; `with_types` validates
extended graphs. Public parameter/result getters now return graph references.
`TypeRef(u16)` is a wire ordinal scoped to one signature, not a frontend `TypeId`.
`Lifetime::Bound(u16)` is a declared late-binder slot, not a serialized `RegionId`.
Unused declared binder slots are retained rather than recomputed from occurrences.

Schema one target/language IDs retain their fixed meanings. Profile zero and one
retain their exact original scalar encodings. Profile two is selected only if at
least one function has a reference node or nonzero binder count. Every function
in that artifact then uses the extended signature encoding, followed by a scalar
constant table whose count may be zero. A profile-two artifact without any
extended signature is rejected as noncanonical. Profile one's constant table
still must be nonempty. Existing function identity/path, Rust/Exact symbol,
unsafe, Rust ABI and nonvariadic prefix bytes are unchanged. Const-body flags and
other conventions remain rejected.

After that function prefix, the extended signature is:

```text
u16 bound_regions
u16 node_count
node[node_count]
u16 parameter_count
u16 parameter_type[parameter_count]
u16 result_type

node = u8(0) u8(scalar_tag)
     | u8(1) lifetime u8(mutable) u16(referent_type)
lifetime = u8(0)                 # static
         | u8(1) u16(slot)       # this signature's late binder
```

Integers are little-endian. Scalar tags remain the fixed values 0 through 16;
mutability is exactly zero or one. Graphs use deduplicated first-use postorder:
visit parameter roots in order, then the result, and visit each referent before
its parent. Every edge refers backward to a lower node ordinal. Unknown tags,
out-of-range roots/slots, duplicate or unreachable nodes, forward/cyclic edges,
noncanonical order and truncation reject before semantic interning. An output
late slot must occur in an input; static output has no such slot. This validates
the declared late-bound profile, not every valid Rust early-bound signature.

Limits are 4,096 nodes and depth 64 per signature, 256 parameters and 256 declared
late slots. Existing 16 MB artifact, 100,000 combined function/value rows,
1,000,000 path/type atoms and 8 MB names bounds remain. Graph nodes, roots and
binder slots contribute to combined interface/dependency budgets. Untrusted
lengths are checked against quotas and remaining bytes before table allocation.
These are logical payload/work limits, not exact allocator or RSS measurements.

## Import and independent evidence

Each imported signature receives one indexed `Vec<TypeId>` remap. Scalar nodes
reuse canonical primitives; reference nodes use their already-mapped referent
and `RegionId::STATIC` or `RegionId::bound(0, slot)`. The complete function binder
is then interned. Existing canonical `ExternalDef -> DefId` remapping and the
source-free foreign Import registry are reused. No AST node, local function body,
initializer or callable execution permit is fabricated. Ordinary call inference,
coercion, well-formedness and loan checks consume the resulting signature. The
unsafe bit stays independent, and ordinary imported functions remain non-const.

`Linkage::import_work` exposes exact signature/node/root visit counts for the one
import pass. At 8/32/128 functions in each of two artifacts, signatures are `2*N`,
node visits `6*N` and root visits `6*N`, regardless of ordinary call use. Opposite
output slots share colliding signature-local ordinals and semantic paths across
the artifacts; reversing artifacts and aliases preserves canonical local remaps.
Only the consumer's actual function has a local body and no initializer is
invented. These assertions bound transport visits, not canonical sorting or total
compiler work.

The declared `borrowed-interface-outcomes.json` contains eleven sources: two
libraries and their valid client, six semantic negatives, and two valid-Rust
early-lifetime profile boundaries. Full pinned Rust 1.98 rlib/object generation
independently qualifies them with edition 2024, overflow checks and panic abort.
Negative Rust error categories are asserted, as are actual RustGM lifetime,
mutability, unsafe, non-const and live-loan diagnostics. The linked native client
checks independent output slots, mutable outputs, nested-reference formation,
shared reborrows, static output, HRTB reification, unit non-null storage, `u128`
values and negative floating zero. Its emitted RustGM objects and independent
Rust program both execute successfully.

Twelve codec groups and all 21 crate-linking groups pass, including existing
scalar/value native groups, reference remap scaling and backend-dependent tests
run explicitly with `--include-ignored`. After final fixture expansion the three
borrowed groups and strict all-target/all-feature release Clippy pass again.
Architecture audit reports 1,396 files and zero findings; layout and upstream
checks pass. Evidence is `obj/parallel/pa29-borrowed/`, especially `fourth-*.log`,
`static-*.log`, `referents.log`, `boundary-*.log`, `qualified-*.log`, immutable
earlier inputs and `source-sha256.json`. Test CLI spelling and a moved test vector
were corrected; failed observations remain recorded. No failed gate is counted
as passing.

Final compiler source digest is
`d373793968984ab4d65c614855b59ec5ab557cd96c2fbc31172cbffdac52ae8b`.
The final source/native/strict receipt is
`obj/parallel/resources/75647ce315804a1399b3ad80acfa4f04.json` (exit 0).

Frozen scalar profile zero (261 bytes, SHA-256
`7b2ba6d33d98a9e930cae29e29da02fbc09ce0a3b41408c65fb1c0cfdc07758c`)
and profile one (722 bytes, SHA-256
`00ffbac14a437de4d12ce92aa933ec5732fdd2d1cc1b0022c7468ea4f6358c3a`)
artifacts compare byte-identically with current production of their unchanged
sources/identities; `scalar-byte-equality.json` records both sides. Fixed protocol
byte tests also cover both original profiles.

Nominal identity/member contracts, generic/const checked body exports, reference
constant allocation graphs, closure metadata and full visibility/reexport
contracts remain unfinished. The existing entry-free packed-pointer startup
boundary remains unchanged. Changed-path wall/RSS measurements await integrator
scheduling; previous unexplained scalar and ordinary RSS observations remain
unresolved. No full matrix, benchmark, oracle generation, push or activation ran
in this worker slice.
