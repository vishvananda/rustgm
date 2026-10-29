# Source-owned promotion declaration marker

These safe and unsafe const function declarations retain the actual `rustc_promotable`
marker, including accepted word/list/value spelling and its unused syntactic payload.
The real NoCore foundation is reused from the previous qualified fixture; these
functions are ordinary authored bodies, not replacements for mem::size_of or SIZE.
Required CTFE and ordinary calls must still execute those bodies and enforce unsafe,
const and API checks. Separate C observation expects 7 8 9.

Pinned Rust accepts returning a static reference to the marked call. That valid
borrow-promotion exit remains explicitly Required in the initial RustGM source-header
checkpoint; no permanent storage or fake promotion is granted. The missing-pair,
nonconst and unsafe-call controls remain language failures. Other const-policy owners
and foreign promotion transport retain their existing explicit unsupported boundaries.

`borrow-required.rs` exercises a runtime `&word()` return, rather than a const
initializer. The marked form is Rust-positive; `borrow-unmarked.rs` retains the
explicit const initializer but removes only the actual `word` marker and fails
E0515. Candidate tests must demand the `borrowed` function body through lowering;
header/source analysis alone is not a promotion observation. Genuine marked
function-call promotion remains an explicit consumer requirement.
