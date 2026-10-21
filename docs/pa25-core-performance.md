# PA25 core performance evidence

Measured candidate `4ebba6418f0d695cc357f665bdc9800835cb4f6e` against baseline
`2e8adf1c8ec18bdc83426c526ea6c4c3d3d63df8`, using immutable release binaries,
Rust 1.98.0, identical frozen inputs and the exclusive shared resource lease.
The isolated baseline Cargo target was removed after pinning its executable.
No observations were discarded or repeated to obtain a preferred result.

Binary SHA256:

- Baseline: `431f9de7ab53243e5a852344b09c080b6980dbd6597ea39064950915bb0eff03`
- Candidate: `89712e19635f3e55302e9e2467b1d7976637725dd0a1fa0b4d36d7e9c4ba2f46`

The study retained six warmups, four paired A/A blocks and five ABBA blocks
for three common controls, with three fresh processes per observation. Twelve
candidate-only scaling inputs each retained four observations of three runs.
All 402 process records include elapsed nanoseconds, peak RSS, status and output
hash. Every common output matched exactly. Inputs and pins were checked for
mutation. Short process durations make A/A calibration essential here.

| Common control | Median paired B/A | A/A block range | Median RSS A→B, KiB |
| --- | ---: | ---: | ---: |
| 512 ordinary functions | 1.0077 | 0.9632–1.0632 | 11,584→11,638 |
| 512 scalar constants | 1.0102 | 0.8046–1.1121 | 10,466→11,060 |
| 128 promotions | 0.9459 | 0.9490–1.1119 | 9,824→9,866 |

These observations do not establish a meaningful small timing change or a
promotion speedup. The scalar control's 594 KiB median RSS increase is observed
and unresolved; the study does not separate fixed binary/code-page cost from
per-declaration storage. Root accepted this evidence for the core checkpoint,
with broader release measurements remaining distinct.

All 15 measured sources were independently object-qualified with pinned rustc.
Candidate-only const values also have independent const assertions. The initial
recursive depth 128 input hit rustc's default CTFE frame limit (E0080); its
original input, qualified wrapper and diagnostic remain in evidence. Before any
timing, the positive depth matrix was frozen as 8/32/96. Array sizes stayed
8/32/128. No timing from the rejected input was discarded.

| Candidate path | Sizes | Actual interpreter steps | Peak frames | Peak slots |
| --- | --- | --- | --- | --- |
| Repeated required calls | 16/64/256 | 370/1,474/5,890 | 2/2/2 | 8/8/8 |
| Unrelated struct declarations | 0/256/1,024 | 25/25/25 | 2/2/2 | 8/8/8 |
| Recursive call depth | 8/32/96 | 424/1,552/4,560 | 10/34/98 | 147/531/1,555 |
| Mutable symbolic array | 8/32/128 | 380/1,316/5,060 | 2/2/2 | 36/60/156 |

Every candidate-only input constructed exactly two source function CFGs.
Charged execution-code bytes for calls were 8,304/31,536/124,464; unrelated
widths retained 1,044 bytes; recursive depths retained 1,744 bytes; array sizes
retained 2,272 bytes. These are the core's typed payload charges, not heap
measurements: the subsequent accounting checkpoint expands callable-map and full
template ownership coverage. The interpreter step counters also exclude the
constant-definition index preparation that inspection found was being rebuilt
between completed roots. Scratch reuse and actual preparation counters are a
separate follow-up, rather than being inferred from these linear step counts.

All evidence is under `obj/pa25-core-performance/`: frozen sources and manifest,
qualification diagnostics, build logs, measurement/qualification programs,
`observations.jsonl`, `summary.json` and `work.json`. Shared resource receipt:
`obj/parallel/resources/8be999a2b10e4a059bd6291540867668.json`.

| Evidence | SHA256 |
| --- | --- |
| `inputs.json` | `dbc11659df334bb9e7a45fb8763dd91b02d3d6ef25a309084128c56fafbf6e7c` |
| `observations.jsonl` | `f80ae33735786db64aa2e081bdad2ac390a8d880a0f04cef44896f470be1ed81` |
| `summary.json` | `f05bd8880c880faa5dfe2b68899b208ea6de779517a8ed7ed0434916af3d7f58` |
| `work.json` | `d0d8d4bfd7be0243d5a35052a1553f8fd023fd6b707ff7e6e4e7ce9be431a847` |
