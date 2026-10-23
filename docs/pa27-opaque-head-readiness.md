# Defining opaque heads and deferred applications

The original reference-cell and owned-transfer programs in
`pa27/tests/opaque-byte-outcomes.json` now reach the existing physical codec.
Both reference programs and the `ManuallyDrop` transfer retain their original
bytes and independently declared acceptance/native contracts. The transfer
executes one runtime destructor, observing the byte value 11. The shared-derived
write control reaches its actual permission rejection rather than stopping at a
missing physical head. Historical failures remain in the outcome record and
`obj/parallel/pa27/opaque-string-backing`.

Expression construction now gives `NeedsDrop` the same immutable source
environment retained by its proof template. An unknown non-Copy opaque value
therefore retains possible cleanup while its original CFG is constructed. The
selected executable still needs the exact physical head to select real cleanup.
The hidden layout grants no source Copy, Clone, Sync, lifetime or caller
permission. Programs without opaque declarations retain the existing unscoped
query path and allocate no deferred preparation state.

Only a fully proved original defining function may publish hidden lifetime
choices. The publisher validates the complete original key/source/signature,
universal count and output-link identity, then requires both the actual region
proof and loan proof. For a jointly checked component, every member's exact
borrowed template/contract/proof pairing and declaration identity passes a
complete first pass before any member closes. The existing `Checked` canonical
index supplies those borrowed rows; there is no second proof index or copied
body. Malformed identity diagnostics never index the claimed unvalidated source.
`prove_body`, specialization and paid-parent proof paths remain proof-only.

The existing cold layout-preparation slot also retains applications skipped
while their exact `OpaqueId` is unclosed. Each full `TypeId` is registered once
as the append-only Store cursor encounters it. A checked closure moves only that
identity's waiting rows to the ready queue in registration order. The next normal
preparation drains those rows and the newly appended Store suffix through the
existing capture substitution, cycle checks, physical-head publication and ABI
layout preparation. There is no Store rewind or whole waiting-map scan. Repeated
close and unchanged polling add no queue work or duplicate registration.

Queue growth is preflighted against 100,000 waiting entries, 100,000 ready entries and a
64 MiB conservative retained/scratch bound. Preparation charges actual visits,
substitution and layout work, with a 32-million-step ceiling. Vector/map capacity
estimates include simultaneously retained waiting, ready and detached replay
storage, roots, pending heads and cycle paths; this does not claim to account for
the entire canonical Store heap. A failed wake or preparation is sticky: a
previously checked closed head cannot turn a failed, unreplayed queue into a later
successful close or execution.

The ordinary `Opaques` record remains 144 bytes and its optional boxed slot
remains 8 bytes. The allocated cold preparation header is 176 bytes, replacing
the former 64-byte layout-template payload. No `Frame`, `Value`, `Origin`,
`Pointer`, ABI shape or source-visible type representation changes.

Focused controls include a fully checked valid-first/foreign-source-second
component, exact original header mutations, caller-payment and escape negatives,
sticky wake exhaustion, preallocation limits, stable registration/wake order,
128 unchanged polls and distinct unrelated waiting identities. Wake work is
`width + 1` for widths 8/32/128 independently of unrelated widths 0/32/128.
Existing codec controls separately cover current initialization/reference duties,
distinct full applications and shared-head width/repetition work. This is logical
work evidence; combined release and comparative measurements remain centralized.

The exact source/native receipts are under
`obj/parallel/pa27/opaque-source-cleanup`. Earlier scalar-only test setup and the
forged-source diagnostic panic are retained as failed observations; the corrected
component control requires an ordinary error with all heads still unpublished.

Focused qualification passes 20 private invariants, the seven original
readiness/caller/escape controls, 16 permanent opaque source/scaling cases across
all three views and 11 literal native comparisons. Strict Clippy, the architecture
audit and upstream checks pass. Of 8,362 active text cases, 8,360 match the old
checkout's references and two match the already reviewed public `7c836691`
oracles exactly. The 98 Python harness tests and suite inventory pass; the old
checkout's layout checker rejects an inherited PA6 receipt base. Corrected
reference/layout qualification and the combined release remain integration gates;
worker references and harness policy are unchanged.

String words and UTF-8 validity remain separate required work. Missing or
unsupported defining heads still reject; an inferred type, layout coincidence,
specialized proof or structural-cache hit cannot substitute for the original
publication authority for these unclosed results. This closes the retained four
readiness cases, not PA27 or PA28 as a whole.
