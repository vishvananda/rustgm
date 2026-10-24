# Const-trait default defining context

The root `contract.json` preserves eleven hypotheses declared before the pinned
Rust 1.98 oracle ran. Four acceptance hypotheses were falsified: an unused
default body calling a nonconst helper rejects, as do ordinary-runtime,
const-unused and const-runtime variants. All eight metadata/object diagnostics
are retained. There were no ICEs or infrastructure failures. The three accepted
runtime/const observations returned native 29.

`provider/contract.json` declares a separate thirteen-input no_core/source-role
context before execution. All metadata/object outcomes matched; six accepted
native observers returned 29. Those include two required const-default positives
whose execution remains outside the initial compiler correction.

The source correction checks an actual const-trait default body under defining
Conditional demand. It does not make that body executable through an ordinary
implementation. Nested ordinary functions own Runtime contexts; ordinary trait
defaults retain their existing behavior. The initial fixtures use actual const
free helpers and explicit conditional requirements. The member family separately
checks the actual default body's derived local Conditional Self premise; it is
not a canonical declaration clause or executable permission.

`members/contract.json` separately declares eight implicit-Self/opposing inputs
in each of std and no_core provider contexts. All sixteen metadata/object
outcomes matched, and eight native observers returned 29. `cleanup/contract.json`
declares an ordinary implementation consuming a destructor-bearing value through
the actual const-trait default. Its pinned Rust metadata/object/native 7 outcome
and candidate runtime/native 7 outcome pass. `recovery/contract.json` declares a
separate unresolved generic-member inference negative: pinned Rust reports
E0282, while the candidate first rejects at a trait obligation. Defining-drop
legality without the conditional Destruct bound remains a separately established
required check; the existing positive cannot demonstrate that negative boundary.
Candidate behavior is not inferred from independent observations.

Evidence is retained under `obj/parallel/pa31/conditional-callable-v1/independent-v1`
for the original hypotheses and `obj/parallel/pa31/trait-default-effects-v1` for
the exact provider profile. Source hashes are in each owning contract; no text
oracles or PA31 activation are implied.

`destruction/contract.json` preserves the five independently settled defining-drop
hypotheses. `destruction/opponents-contract.json` separately declares moved Self,
lookalike Copy and ordinary-only Destruct opposites; all four metadata/object
outcomes and two moved-Self native7 observations match pinned Rust.
`destruction/work/contract.json` freezes distinct cleanup-subject widths
8/32/128 and width8 with 128 unrelated definitions. Its metadata/object/native7
observations all match pinned Rust. Focused development-v9 also passes the
candidate original-proof and native controls; its bounded preparation contract
and retained gaps are documented in `docs/pa31-defining-cleanup-preparation.md`.
No generated text oracle, imported-default acceptance or whole-core claim follows.
