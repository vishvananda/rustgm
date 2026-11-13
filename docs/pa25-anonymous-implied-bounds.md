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
