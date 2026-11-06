# Concrete CTFE cleanup producer

This is an intermediate producer checkpoint for LIB176. It does not execute
destructors in the constant interpreter. The active-cleanup Code fence and
conditional-parent fence remain in place. The separately frozen Rust-positive
`ct41`, `ct59`, `ctonce` and `ctcond` inputs still require execution work.

The source baseline is joined main027a7203d, including the Field/AsyncIterator
consumer changes. The independent packet and original prefix bindings are in
`source-ctfe-reference-evidence.json`; its three Rust native observers return
41,59,41. These are independent expected effects, not candidate CTFE results.

The one-body verifier now has sealed `Prepared<P=symbolic::Plan>` payloads.
Original definitions keep their symbolic plan. Code selects a genuine concrete
Plan from the same initialized Facts and Seal, using the existing glue Catalog
and physical cleanup transfer/replay. Its eligibility-only refinement does not
activate the old physical exclusion semantics. Both payloads use the same
selected-leaf authentication, semantic const eligibility, mandatory region
refresh and ordinary destructor-aware loan completion.

The shared completion kernel borrows the owned Prepared token. Symbolic finish
consumes and drops it; execution finish moves out its concrete Plan and SAME
Seal while retaining the actual Body borrow. A second exact-owner validation
precedes the retained interpreter fence. This result is a loan result, not a
standalone const execution certificate. Code publication still requires the
actual refreshed Contract/State and exact selected full target.

The synchronous method inventory reuses the body::drops method kernel. Code
pays the actual source caller ledger before its temporary inventory allocation.
Catalog recipes and planning allocations retain their existing bounded owners;
the cache is detached before mutable semantic selection, without cloning it.
Recipe/planning work is included in the recorded canonical-work statistics.
The cache is currently released before returning from the producer; an actual
owned executable bundle remains the next consumer requirement.

The existing no-required/no-cleanup guard remains unchanged. A potentially
destructible but physically inactive Code body can now pay transient concrete
catalog/planning work. This checkpoint does not claim zero extra work for that
path or a general performance improvement. Ordinary runtime checking and
Original symbolic selection gain no concrete catalog allocation.

Measured widths are Initial728, symbolic Prepared784 (unchanged), concrete
Prepared904 and borrowed Execution224. Code304, PreparedOperand32, Frame248,
State424 and Machine4720 are unchanged. The three new producer widths are
compile-evaluated guards. No Body, persistent Plan, wire or backend field changes.

Qualification is bound in `source-ctfe-concrete-evidence.json`. The focused gate
covers46 exact public tests, all eight old cleanup groups, three existing native
observers, strict Clippy, formatting, architecture, suite registration and pinned
upstream verification. The native observers remain ordinary runtime41/47/59,
not evidence of candidate destructor CTFE. The sparse array work control records
equal work for lengths8 and1048576; ordinary cleanup scaling remains bounded.

The first strict attempt found one unused import. A later test used only
analyze_source and consequently did not demand an unused invalid const Drop
method body; its complete-lowering preflight now observes every original
negative independently. Real rejection causes remain const Destruct, non-const
call, unsafe dereference and live loan. A borrowed-completion refinement then
needed two explicit consuming token drops for strict Clippy. All failed logs
and source maps remain in obj/parallel/source-transport/ctfe-exec.

Next execution work must retain the concrete Plan/recipes/timeline in a paid
owned Code footer, bind it once to the exact current Rc<Code> through a Weak,
authenticate real selected const Drop bodies at reached events and return their
actual Unit results to an exactly-once cleanup continuation. A plan or owner bit
cannot supply those permissions. First execution remains a finite closed sized
Struct/unconditional cleanup profile; conditional flags, generic/conditional
parents, recursive fields, arrays/enums/slices/dynamic/future cleanup, broader
provided bodies and full source policy/metadata remain required work. Private
hostile execution and complete release checks belong to subsequent CI; none is
claimed by this producer checkpoint.
