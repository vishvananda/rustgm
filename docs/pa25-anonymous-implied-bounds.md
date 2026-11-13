# Anonymous constants and enclosing implied bounds

The unchanged original-core observer on release16 clears the initializer-scope
failure and checks42310 declarations and25858 bodies without a failed function.
Completion then fails while proving an original anonymous constant generated
from slice/iter/macros.rs. The diagnostic span anchors at slice/iter.rs EOF.
The debug16c stack identifies check_original_child through anonymous_needs_parent;
this is an original-child region proof, before concrete execution.

Eight independently accepted reductions reproduce the failure in release16:
ordinary/source wrappers, direct/macro-generated inherent/trait methods. The
struct requires `T: 'a`, while its impl omits that redundant explicit bound.
Their explicit-bound controls accept. The anonymous contract retains explicit
predicates but omits the enclosing receiver's implied well-formedness bounds.

Anonymous headers now reuse the existing creator-signature implied-bound cache
shared with closures. The inherited universal prefix must contain the creator's
declared slots. Both batch and incremental contract collection receive the same
cached scalar/type assumptions. The anonymous body gains no callable signature,
local-storage permission, projection proof or fabricated constant value.

The [PA25 packet](../pa25/tests/spec/anonymous-implied-bounds/README.md) and
[PA31 packet](../pa31/tests/spec/anonymous-implied-bounds/README.md) each declare
twelve outcomes: eight accepting and four rejecting. All24 agree with Rust.
The selected contract contains eleven per packet, retaining the original
borrowed-result native consumer as a required twelfth positive. Its full gate
first found the output transport boundary, then an unclosed result-lifetime
connection. No acceptance is inverted or omitted from the manifest.

Actual independent Rust/C borrowed and scalar-result programs print7. The source wrapper retains its
original methods and the existing real panic_nounwind hook for runtime validity
assertions. Static/local escapes, unpaid static bounds and unrelated universal
regions remain rejecting contracts. Source inputs, expectations and observer
bytes were frozen before execution; failed native consumer setup bytes remain
in native-declaration-v1/v2/v3 with their actual Rust diagnostics. A mismatched
scalar C return declaration failed oracle5; oracle6 uses the exact u8 ABI and
passes both scalar observers. Original observer bytes remain intact.

Callable anonymous results now enter the existing requested-result transport
and complete reproof path, as source initializers already do. This retains every
child constraint and the actual requested-result coercion. It assumes none of
the original parent's exported conditional conclusions. The borrowed-result
consumer still requires exact enclosing lifetime connections.

The selected22 complete RustGM outcomes pass, including all eight opponents.
The separate batch-contract consumer passes after the ordinary template CFG and
promotion preparation. fast3 passes six exact semantic/scaling/previous controls;
fast4 receipt1794607901095329290 qualifies that final batch consumer plus strict
selected/CLI Clippy, full formatting and architecture audit in44.08s. Failed
fast1/2/3 gates remain: the native borrowed-result boundary, then two incorrect
batch consumer setups that lacked parameterized CFG/promotion facts.

The current hash-checked harness executes both scalar native programs printing7,
the earlier three ordinary native programs printing7 and the actual source
five-export observer printing4 8 4 4 8. At8/32/128 owners, inference is
528/2112/8448 =66n, canonical80/320/1280 =10n, anonymous templates exactlyn.
The four exact frozen measurement controls independently accept in Rust; paired
same-profile timings remain pending behind the unchanged14/20 load gate.

Raw evidence is under inline-region-arrays/. No new original-core observation
of this repair or original artifact is claimed. Whole PA25 and PA31 remain planned;
the existing113-case PA25 component is unchanged.

## Late creator publication

Clean release17 on d1e15f2ad retains the same original completion failure.
Its postchecked120s diagnostic still has42310 declarations/25858 bodies,
35120 item attempts and no failed/blocked functions. All722 original hashes,
375 opens, six pins and compiler limits remain unchanged. No original artifact
or complete analysis is claimed.

The debug17b trace identifies the exact first Iter::next_chunk anonymous body,
NodeKey227:9558, and shows creator26576 missing from the early signature snapshot.
Four smaller function-signature array lengths reproduce the ordering: an earlier
constant demand warms the nested-bound cache before later headers are published.
All four accept in Rust and fail release17. Nominal-field completion controls
accept; they do not exercise the same early publication order.

Creator lookup now uses the existing declaration index on each request and keeps
only computed implied requirements in its cache. It preserves bodyless/async
exclusion and does not cache missing owners or scan the whole function inventory.
The packets retain14 independent outcomes each;13 selected complete outcomes
pass, with the original borrowed-result native consumer still required.
Receipt1794609319805878213 passes seven exact tests, including actual captured
const-closure values and lifetime opponents, plus strict selected/CLI Clippy,
full formatting and architecture audit in134.87s. New original verification
and paired immutable timings remain pending.

The current-source/harness-hash checked cache-native replay passes both scalar
programs and all earlier anonymous native controls; owner counters remain66n/10n/n.
Fresh128+126 upstream checks pass. No backend contract or original source changes.
