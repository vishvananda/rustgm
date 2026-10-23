# Scoped actual parent proofs

The measurements below qualify the original `73353ab9` checkpoint only. The
frontend-performance adaptation and its focused qualification are described in
[the port record](pa25-parent-perf-port.md).

The unchanged `inline-query-context/closure.rs` and `paid-lifetime.rs` positives
now complete checked constant execution and LowIR. Their original component proof
is still mandatory and grants no instance execution authority by itself. This
checkpoint supports ordinary lifetime-only function/closure parents. Generic,
SourceInitializer and further nested conditional parent families remain required
PA25 followups; no ordinary anonymous or Associated concreteness gate is relaxed.

The solver pairs each actual immutable Contract with its region result and one
ordered Arc lineage. The ordinary ownership, storage, destruction and loan
verifier produces the actual Proved rows once. CompleteGroup checks every exact
row, ordinal, BodyKey/source and shared lineage before lending an indexed
ParentMember. Failed ancestors, missing/duplicate/foreign rows and stale owners
cannot activate a child. Context-only originals never become runtime candidates.

Each actual parent retains its exact inline operands, full InlineInstances,
requested results, registry context and declaration epoch, plus the fold-produced
bridge-slot map. Checked original Summary requirements are added as parent goals
and fully proved. Only the live actual parent can lend those exact formal
conditions to the child. The child retains its OutputLink and reproves all
ordinary obligations, actual-to-requested result coercion, eligibility, promotion,
storage/escape and loans before Code publication. Nothing is reconstructed from
matching shape, arity or source text.

Conditional parents use a cold Rc guard around the one actual Proved. A borrowed
complete group keeps all ancestors/siblings alive during demand; final append
requires unique extraction of the same owner. No proof graph or Proved is cloned.
Ordinary programs keep their prior proof and constant-demand order and do not
allocate this sidecar. Conditional-only staging boxes ordinary paired rows to
keep its tagged entries small; the ordinary Builder proof vector is unchanged.

The executor parks the exact prior roots, sessions, completed results and
suspended storage, then restores them on success or failure. Nested cold Code
indexes require the exact live scope; retired entries cannot start a machine.
An exact-key value journal removes only publications owned by a failed scope,
preserving prior materialized values and independently completed nested values.
Materialized values after retirement are not execution permission. Limits cover
combined parked state, Code/index/journal capacity and actual port/debt traversal
before growth, retaining failed work. Retirement visits only the scope's own keys.

The private matrix checks actual ancestor failure, complete/ordered group identity,
epoch/body/instance/result/operand rejection, nested index/journal restoration,
foreign writes and quota rejection. Actual successful and faulting two-port source
programs restore all prior caches; the faulting child retains the arithmetic
rejection and publishes no callable-owned result. The prior wrong setup reading
a nonexistent local, exact arithmetic-wording failure, filename audit failures,
and initial aggregate-work ceiling hypothesis remain in the evidence directory.

For 1/8/32/128 actual ports, preparation visits are 5/47/191/767 and retained Pending
payloads 536/1232/4592/18032 bytes. Full pipeline canonical work is 1873/13017/51225/
204057, exactly 1592 additional visits per port in this family. This is logical
work evidence, not a timing claim or an attribution of all pipeline work to this
change. Code retains 3376/24488/96872/386408 bytes; all exact cache roots/sessions
are restored after demand. The initial 1000-visits-per-port aggregate ceiling was
wrong and is preserved; the final tests assert exact preparation accounting and
affine increments across the unchanged four inputs, without retiming.

Measured layouts are Engine 2152, Code 336, Proved 656, Body 464, Value 24, Target 28
and Request 56 bytes, unchanged from the original-component prerequisite.
Cold additions are Active 208, OwnedParent 8, PairedProved 680, Port 88 and Pending 112.
The retained original-component row gains one optional 8-byte Active pointer.
AnonymousIndex and inline Values each gain one optional 8-byte cold index/journal
pointer; existing nonconditional anonymous use can therefore retain that small
header increase. Code cloning and ordinary Proved ownership remain unchanged.

Closing evidence is `obj/parallel/pa25-paid-parents/close-third/`, together with
its `close-second/` private group/port/journal/index logs. The production source
map is unchanged between these tails. All 23 relevant private tests passed,
alongside 17 selected-initializer tests, both legacy caller/lookup suites, eight
native inline programs, seven independently emitted Rust objects/status controls,
strict all-target/all-feature release Clippy, the 1,847-file audit with zero
findings, registration, layout/upstream checks and all 8,362 archived PA1–24 text
comparisons. This is the retained fixture archive of 11200c5, not a claim of
central qualification against newer public production or frontend-perf changes.

The immutable CLI pin is `close-third/pins/rustgm`, SHA256
`bb7a12b3addc82fb8ea33cddfe908ff7d2e8f644566dc14ab548fe5aa22a67e6`.
Its 1,057-file production map hashes to
`133dcfbfcdb19a977a0591ce2f61df4ea1332acadaf7e3e03f959c00f58170bf`.
Terminal resource receipt
`obj/parallel/resources/2ce193a331ca4c2590e207829df559a3.json` reports return 0,
30,848,684,032 observed generated bytes under the unchanged 50 GiB worker quota.
No references were generated and no assignment activation is claimed.
