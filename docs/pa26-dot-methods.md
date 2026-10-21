# PA26 canonical trait dot methods

Already demanded canonical operator and assignment traits now participate in
ordinary dot lookup. The visibility index consumes the retained declaration
member inventory, including explicit language members, and admits canonical
traits only through their registered operation identity. Method spelling enters
the existing name filter when the language contract is prepared. No generated
source, spelling-based trait recognition or second dispatch path is involved.

The canonical trait owner retains the same implicit `Self: Trait<Rhs>` predicate
as an ordinary trait declaration. Calls therefore infer their complete trait
application before scalar fallback, including unit-returning assignment methods.
Bound-selected methods use the ordinary assumption proof. Receiver candidate
substitution includes the enclosing Self parameter as well as trait arguments.
The primitive negative-head filter recognizes only the already registered
assignment domain; full selection still checks exact operand pairs and retains
the previously validated primitive method descriptor and ordinary body proof.

Scope/name lookup shares one declaration-member index and inherited candidate
lists. `Methods.trait_visibility_steps` includes initial index construction and
cached lookup work; existing receiver probes and inference work remain charged.
Widths 8, 32 and 128 pair repeated canonical calls with distinct unrelated method
names. Visibility work is 113, 353 and 1,313, exactly ten extra steps per width;
there is one receiver probe per call and fewer than 100 inference steps per call.
These are bounded algorithmic controls, not a cumulative performance release.

The draft matrix has 26 independently qualified outcomes: 17 accepted programs
and nine Rust rejections. Accepted programs cover nominal and primitive receivers,
shared RHS forwarding, generic bounds, aliased/anonymous/glob/block imports,
lexical shadowing, inherent priority, user lookalikes, shared/mutable indexing,
owned Add, captured calls, two-phase reservation and receiver-before-argument
evaluation. Opposing cases cover missing imports, module scope, ambiguous traits,
conflicting loans, immutable receivers, moved receivers, unsupported primitive
pairs and unrelated nominal receivers. Every accepted program has native value
checks against pinned rustc and the configured external LowIR backend.

Evidence is retained under `obj/parallel/pa26/dot-methods`. The initial gate
recorded rejected canonical receiver matches; the second recorded missing implicit
Self obligations for unit-result methods. Both failures remain separate from
the successful third source/native/scaling gate. The independent second gate
qualified the first 24 controls; the next independent gate qualified all 26.
The first regression gate found the old IndexMut dot-call boundary now accepts;
its unchanged bytes retain independent Rust qualification and now join the
accepted source/native controls.
Earlier rejected test hypotheses (module scope and a wildcard pseudo-read after
move) are preserved and were corrected before compiler outcome qualification.

The final functional gate passed 137 source test groups across language operators,
ordinary dot methods, method selection, trait bounds and call operators. All six
language-operator native groups passed, including the unchanged IndexMut program
whose direct-method boundary now accepts. Strict release all-target/all-feature
Clippy passed after replacing single-letter names in the new scaling assertion;
only that affected invariant was rerun. The architecture audit covered 1,338 files
with zero findings, layout covered 38 stages, and upstream verification covered
24 pinned fixtures and 126 Reference files. Results are in `final-second.json`
and `final-third.json`; the earlier failed gates remain recorded separately.

Implicit canonical hook availability, other primitive operators, comparison and
Deref, iteration, question-mark/residual flow, additional derives, formatting and
runtime/library contracts remain PA26 work. This checkpoint changes neither
activation nor references. PA30 aggregate ABI classification, variadics and its
remaining metadata/release integration are also separate unfinished work.
