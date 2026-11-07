# Const closure original component reuse

Reached const closures now retain their original in the existing indexed source
component owner. Repeated selected instances borrow that checked original and
consume a normal materialization; the post-materialization dispatcher, region,
loan and Code publication path is shared with ordinary function specialization.
There is no new template registry, permission bit or hot record field.

Component publication retains both the checked contract and the displaced source
contract. Each needs genuine Copy and region proof against the same actual body.
The Copy preparation kernel now accepts these inventories separately, sharing one
body index. Duplicate source rows fail; primary rows still consume exactly one
body each and retain their exhaustive identity checks. New source-row scratch
growth and failed attempts are paid before allocation. Ordinary source-free rows
without defining requirements keep the lazy allocation-free path.

The first gate's two capture failures are retained as
`1794019361736767587`. After the inventory repair, gate
`1794019763195241345` passes ten exact public tests in 113.79 seconds,
including the full captured CTFE pipeline and loan/move/lifetime opponents.
The identical production fingerprint passes the five existing native contracts
in `1794019898948626366` (23.94 seconds), producing
`closure-source 23 47 23 23 47` from runtime input 23.
The JSON companion binds every receipt and the production fingerprint.

Five new private fault controls cover duplicate source/primary inputs, a missing
primary contract, foreign source identity and a changed signature. Their test
compilation/execution and prior sealed-environment adversaries remain pending
metadata/full CI; focused public checks do not establish those private results.
Generic and conditional defining obligations, proved receiver adapters,
bootstrap const calls, wider binder transport and full capture destruction remain
required parts of the ongoing const-closure implementation.
