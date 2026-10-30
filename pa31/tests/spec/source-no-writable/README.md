# Source LLVM Writable suppression

Pinned `rustc_no_writable` accepts a word on a real free function, inherent or
trait-impl method, default trait body, or closure. It requires `rustc_attrs`
(or the actual macro allowance) and rejects duplicate markers even under lint
allowances. Required trait declarations, foreign functions and other targets
reject. Closure syntax separately needs its real expression-attribute feature.

The marker only suppresses the optional LLVM Writable argument assumption. It
does not prohibit stores through `&mut`, make `&` writable, grant unsafe/const/API
permission, change ABI, or imply `inline(never)`. The current RustGM and locked
native backend emit no LLVM Writable assumption for local or imported bodies.
Source validation is therefore the current semantic implementation; there is no
persistent absence query or new wire field. Any future optimizer consuming that
intent must first preserve/authenticate it across imports or remain conservative.

`contract.json` freezes the authored independent outcomes before execution.
`primary-map.json` identifies the exact pinned parser and ABI consumers. The
unchanged original `index_mut` method bytes are retained separately; its full
IndexMut/SliceIndex const/generic/body prerequisites remain required. That text
is not a candidate or independent standalone-body qualification claim.

The frozen41 inputs match all82 pinned Rust metadata/object predictions. The
[joined source checkpoint](../../../../docs/pa31-advisory-source-checkpoint.md)
now passes the new controls and existing MIR-inline/Has/coverage controls,
production Clippy, affected text suites and actual candidate/Rust native7.
Real owner/payment/layout, const/unsafe/borrow and independent inline checks
remain intact. Original method graphs and broader assignments remain required.
