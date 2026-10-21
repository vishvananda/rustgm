# Typed receiver and array capture uses

PA24 remains planned. This producer extension consumes selected method receiver
borrow facts and treats array indexing as capture of the complete array. The
index expression retains its own uses. Field reads through an indexed non-Copy
aggregate use the leaf's Copy permission, without moving the enclosing aggregate.
Copy queries are deferred until traversal reaches an imported place, so unrelated
local projections do not force premature inference defaults or permissions.

Ordinary function items, constructors and function pointers follow their existing
checked call signature. A missing callable-trait invocation record is accepted
only for those typed constructors; other callable families still require their
selected receiver record. This also fixes a captured destructor value passed to
an ordinary generic function.

The draft declarations in `pa24/tests/capture-use-outcomes.json` cover nineteen
positive inputs and four borrow/move rejection controls. All twenty-three
outcomes match independently compiled pinned rustc objects. Five additional
value-checking native programs have independently accepted object outcomes in
`capture-use-native-outcomes.json`. These are not public text oracles.

The initial fifteen positive semantic capture checks pass, along with the
existing closure, inline-constant and raw-read regression targets. The final
producer source passes strict release all-target/all-feature Clippy, formatting,
the file audit (1,321 files, zero findings), layout and upstream checks. Evidence
is in `obj/parallel/capture-uses/`: `gate-v1` records the initial source tests;
`gate-v2` retains the filename-ownership finding corrected by naming the prefix
scaling test `prefix_growth`; `gate-v3` records the final static checks.

The final direct-call/local-projection checks and prefix-growth invariant now
pass with the closure lifetime regressions: twenty-seven source test functions,
recorded in `obj/parallel/capture-uses/source-v2/`. For 8/32/128 distinct fields,
retained path counts are 9/33/129 and graph work is 42/162/642; total inferred
body work is 1,600/5,632/21,760. Repeated use of one field retains two paths and
one capture, with graph work 28/100/388 and body work 641/1,385/4,361. Both
series have constant added work per added source use over the measured widths.
These counters establish source scaling; they are not wall-time measurements.
The five new native programs await the combined capture-execution gate. Adjusted
receivers, reference/slice or overloaded indexing, pattern and nested capture
propagation remain separate work. The unadjusted source access map retains its
original meaning; a future adjusted-access contract must preserve the exact
selected dereference prefix and ordinary borrow/unsizing proof.
