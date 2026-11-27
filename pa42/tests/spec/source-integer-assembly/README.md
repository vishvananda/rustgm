# Integer assembly and flags transport

The outcome manifest declares five independently observed native results before
reference generation. The original x86-64 `__readeflags` and `__writeeflags`
functions are inserted byte-for-byte from `../../upstream/rust/eflags.rs`, pinned
to Rust 1.98.0 commit 88d9e12ae178fab0fb5cc050a94da85685d449ea. See
`third_party/sources.json` and `shared/RUST-NOTICE` for provenance and licensing.
The foundation reuses `../asm-native/native.rs` before its SIMD declaration.

The authored generic functions exercise fresh v14 consumers, inout, lateout,
aliased outputs, reference destinations and AT&T order. The separate runtime observer checks EFLAGS reserved bit 1, avoiding values of
flags Rust leaves undefined on entry. This remains an unqualified expected-success
acceptance: pinned CPPGM v0.10.3 does not encode `pushfq` or `popfq`. The original
functions export and import with their exact AT&T templates; no instruction is
rewritten to bypass the backend limitation.
The older-profile and safe-call controls remain explicit semantic rejections.
