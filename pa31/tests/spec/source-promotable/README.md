# Source-owned promotion declaration marker

These safe and unsafe const function declarations retain the actual `rustc_promotable`
marker, including accepted word/list/value spelling and its unused syntactic payload.
The real NoCore foundation is reused from the previous qualified fixture; these
functions are ordinary authored bodies, not replacements for mem::size_of or SIZE.
Required CTFE and ordinary calls must still execute those bodies and enforce unsafe,
const and API checks. Separate C observation expects 7 8 9.

Pinned Rust accepts returning a static reference to the marked call. The header
promise itself grants no permanent storage; the promotion engine promotes that
borrow by executing the marked call (see `../promoted-calls/`). The missing-pair,
nonconst and unsafe-call controls remain language failures. Other const-policy owners
and foreign promotion transport retain their existing explicit unsupported boundaries.

`borrow-required.rs` exercises a runtime `&word()` return, rather than a const
initializer. The marked form is Rust-positive; `borrow-unmarked.rs` retains the
explicit const initializer but removes only the actual `word` marker and fails
E0515. Candidate tests must demand the `borrowed` function body through lowering;
header/source analysis alone is not a promotion observation. The private guard
lowers `borrowed` from both: the marked borrow is promoted and the unmarked one
is rejected. `contract.json` records the files at its revision.
