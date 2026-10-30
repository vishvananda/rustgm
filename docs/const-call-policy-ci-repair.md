# Ordinary const contexts and generated call policy

Full qualification40976018856 on788889aac passed2366 private controls and all
but one integration outcome test, exposing two regressions from the recent
compile-time-only policy work. The target-feature const-missing case reported
missing source coverage before its real unsafe/feature check. The integer-range
bound-selects-usize positive tried to recover a single source Call from a For
node. Neither fixture or expected outcome is changed by this repair.

Target-feature declaration checking may create the shared header budget after
the declaration-attribute pass completed. A newly allocated budget is not a new
unfinished attribute pass. The declaration checker now closes that already
completed phase before scheduling an early const body. This needs no new field,
source scan, per-function allocation or unknown-policy waiver.

Iteration already retains separate into_iter and next callables. Check each
through the existing typed API while preparing its complete For plan, before
publication. The later ordinary-call loop leaves that plan intact instead of
interpreting its node as source Call syntax. Canonical Option owner/variant
lookup is one explicit typed operation, keeping functions within Clippy limits.

Canonical factory methods have no authored function header. Their existing
authenticated registry/kind/scope/binding/import checks establish the absence
of an authored comptime marker, without creating a source contract or granting
execution. Real imported callables still require their actual header. A forged
source function with a removed resolver node now fails the declaration-origin
check instead of using the imported-absence path. Existing malformed/missing
factory opponents and genuine factory/source positives remain enforced.

## Qualification

The repair passes six private tests with strict Clippy, formatting and full
architecture checks in135.56 seconds, including the actual target-feature
regression, real Always positives and changed/missing/forged owner opponents.
Receipt8fa7e09c194dcb314e2a065ac3ca1fd92663a974e50d8e9495b47d6f18f2690c.
The exact production integer-range outcome test passes all47 declared Rust
positive/negative inputs through checked LowIR in100.97 seconds, including
strict production Clippy and static checks. Its receipt is
610362b6cde3dce5856e85a289f1f4db7d459e0234a02f63d277f05944acd148.

The initial mixed cold gate completed strict private code generation but reached
the five-minute limit building the separate integration target. A warm retry
then exposed the missing canonical-factory absence route, which is repaired
above. Raw static/type/budget and semantic predecessor failures remain in obj;
none count as runtime qualification. Private and production validation now run
as separate bounded development gates. Caps, deadlines, fixtures and expected
outcomes remain unchanged. Full qualification will be repeated on the repair.

The latest actual whole-core observation remains CLI49f16ba0 at the original
reservation attribute in convert/mod.rs801. This repair alone does not establish
a new original-core observation, complete a PA or produce an original artifact.
