# Source arith_offset checkpoint

The actual `arith_offset` source declaration now supports checked direct calls
through the source catalog, runtime lowering and constant execution. The
independently frozen actual caller, distinct ordinary lookalike and generic
`at<T>` forwarding caller all produce the same respective native traces through
the existing backend as pinned Rust. This closes the finite source intrinsic
consumer, not whole core or source metadata transport.

`SourceIntrinsic::ArithOffset` is appended without changing old variants,
bootstrap `Intrinsic` factories, LangItem tags or wire profiles. Its declaration
retains the actual owner, attributed source header and validated signature. A
private descriptor checks the full closed one-type instance and exact
`(*const T,isize)->*const T` substitution only when selecting execution; generic
original body typing remains parametric. A checked source caller/Code supplies
execution authority separately. Ordinary unmarked functions keep their bodies.

Runtime reuses the existing signed machine-word multiply and typed byte-index
kernel. CTFE keeps numeric raw/null words numeric and normalizes symbolic offsets
to the actual original allocation root, preserving each pointer's write
restriction. Wrapping and zero-size arithmetic do not inspect storage or assert
object bounds. Existing accesses still enforce bounds, initialization, validity,
liveness, alignment and write permission. Existing inbounds `add` behavior is
unchanged. No Body, Code, NodeFacts or catalog field was added; measured seals
remain Body456, Node36, NodeFacts80 and Catalog1528. Repeated projection controls
at widths8/32/128 retain a single root with work bounded by3×width; failed zero
budget retains the attempted visit.

The final private gate is
`obj/qualification/fast/1793252221157288554/receipt.json`, SHA-256
`5040ce14f7b6ce66d6ece970368b629915b307523ca2db2ea4f87919c99505b6`:
format/architecture/strict-private/build and eight exact controls passed in
104.25s. The controls include actual and generic source consumers, CTFE and
reached invalid dereference, signature/generic/unsafe/nonconst opponents,
foreign-report/instance/header/non-call forgery, pointer restrictions/failure
work/scaling, and the existing inbounds evaluator and byte-index shape tests.
Production/CLI and the joined actual Debug observer remain integrator checks.

Independent Rust v3 receipt SHA-256 is
`ebcbed585a3a1b2d8b58c24345458765a96cf203d22b7967581e98cf233c97ac`:
eight metadata outcomes and three source/staticlib/C/native traces matched.
Candidate native receipt
`obj/parallel/wrapping-offset-v1/native-observed-v1/receipt.json`, SHA-256
`5bf555fa136c61d8d0eb192554b4ec7c5a1f0c50e3dc321426ffa6328eb846ee`,
binds the passing private producer, immutable backend/tool hashes and all three
exact caller source/LowIR artifacts. All twelve validation/object/link/run
operations passed with exact stdout/stderr/status equality.

The first Rust hypothesis omitted the real inherited const-policy feature and
is retained as a failure. The first candidate gate stopped at strict lint; the
next passed6/8 but two direct kernel controls used an intentionally unresolved
report helper. The final retry uses genuine completed resolution. None of these
failures was treated as a language or execution pass. The contract's explicit
byte-range erratum retains its predecessor binding: the fragment hash and all
observed Rust/C source bytes are unchanged, and the pinned original verifies at
bytes37329..38155.

Source declaration and body publication explicitly reject this intrinsic until
its semantics/provenance have a supported metadata profile. CTFE literal-text,
function and wide-pointer representations remain required. Full original
library/runtime acceptance, wider artifact closure and optional optimization
behavior remain separate requirements; this checkpoint adds no pointer-access,
const-feature or noalias permission.
