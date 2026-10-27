# Higher-ranked const trait bounds

LIB-040 is the actual original core parser boundary at slice/cmp.rs218.
PA5 owns bound grammar, PA21 binder relationships and PA31 the unstable const
feature authority. The pinned parser reads the higher-ranked binder before const
modifiers; the current baseline does the reverse. Original upstream code remains
unchanged. Nine source cases freeze eighteen independent Rust metadata
observations and eighteen immutable RustGM baseline observations across2021/2024.
All declared statuses match; no runtime or whole-library claim follows.

The contract includes a conditional bound, an always-const bound, an impl Trait
parameter, an ordinary higher-ranked control, reversed-order opponents, a relaxed
bound, a modified lifetime bound and a missing-feature opponent. Retain the actual
binder list and Constness in the existing Bound node and use normal semantic and
feature consumers. Grammar acceptance grants no const trait implementation.


Integrated source `ef9cf4635` passes both strict Clippy modes, architecture,
33 focused tests, all231 PA5 text cases and all758 PA21 text cases in118.93s.
The first private lint finding and the two caught rejection-location changes
remain recorded; existing references were not edited. The qualified CLI is
`d1e2137d62edbbd9ad44d54e7e15816648ab80ef71721b7771685ac08205b628`.
Original core passes this boundary, opens263 files and stops at tuple.rs31's
`${ignore($T)}`; all722 original pins verify unchanged in3.00s. Evidence lives
under integrator `obj/parallel/higher-ranked-const-bounds-v1/`, with producer
receipt `1793082428623066513` and explicit CLI/source configuration binding.
Actual generic header/body checks and the missing-feature opponent pass in both
editions. Broader callable/const execution remains owned by the normal source
protocols; no native execution or complete library artifact is claimed here.
