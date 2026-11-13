# Anonymous associated-value borrows

Original generic inline constants keep associated values deferred until the
actual caller supplies their instance. This repairs the reduced form of original
core's completion failure at mem/mod.rs1421: promotion tried reading a generic
SIZE default without the parked initializer caller. Actual core verification of
this checkpoint remains pending; no original artifact or whole PA is complete.

Promotion uses the existing original-owner policies and the registered anonymous
body's DeferredTemplate state. An anonymous original/instance being prepared by
the source constant engine lends dependent value requests back to ordinary
execution; it grants no constant value or trait/lifetime permission. The provider
is restored after preparation, including failure. Bootstrap scalar evaluation
retains its existing path. A closed expression now collects its contract and
resolves actual associated implementations before promotion reads their values,
matching the already established initializer/anonymous instance ordering.

The [ordinary PA25 packet](../pa25/tests/spec/anonymous-associated-borrows/README.md)
has five outcomes, three accepting and two rejecting, independently declared
before first execution. All pass complete preparation. Three actual Rust/C and
RustGM/C programs each print7. Unpaid bounds and non-const calls retain their own
rejections. Clean release15 rejects all three positives before this repair; raw
observations remain, with no expected positive inverted.

The [PA31 packet](../pa31/tests/spec/anonymous-associated-borrows/README.md) binds
18 independent source outcomes,16 complete prepared outcomes and one additional
projected source-pipeline acceptance. Two complete consumers remain required:
projected layout argument normalization, and concrete inline source promotion
value dependencies. The former also fails release15; the latter moves from
reentrant eligibility to an explicit deferred-promotion boundary. Neither is
classified as a negative or omitted from the manifest.

Five actual source Rust/C and RustGM/C exports print4 8 4 4 8, covering distinct
u32/u64, tuple and required const-call instances. The source layout defaults are
real initializer bodies, not replacement values.

Receipt1794603811953549205 passes six exact source/ordinary, source projection,
scaling, existing MaybeUninit and structural-pattern controls in136.28s. Selected
production/CLI strict Clippy, full formatting and architecture audit pass.
Hash-checked final-replay executes both native tests and the scaling assertion
without recompiling. At8/32/128 owners, inference work3521/6545/18641 is126n+2513;
canonical4473/6585/15033 is88n+3769; original anonymous templates are exactlyn.
All exact borrowed and unborrowed families independently accept in Rust.

All315 Python harness tests,488 integration roots/13 binaries,44-stage layout,
128 upstream fixtures and126 Reference files pass. No reference, upstream byte,
backend contract or compiler limit changes. No dense type/body/syntax field grows.
The paired immutable timing comparison and new original-core observation remain
pending; work counters do not establish wall-clock performance.

Raw failed gates are retained: fast1 exposed a pre-existing projected artifact
gap; native1 exposed general evaluation during active template promotion; fast2
failed the function-size lint; fast3 retained two complete consumers; fast5 found
the ordinary closed-expression ordering. All source/native predictions remain
unchanged. The brief read-only Btrfs incident was recovered externally, and the
verified recovery archive still preserves the pre-recovery state.
