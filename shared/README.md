# Shared contracts

`lowir.md` is copied unchanged from the CPPGM commit in `backend.lock.json`.
It defines the temporary backend boundary. `CPPGM-NOTICE` retains inherited
attribution. Rust lowering gets its own output conventions; C++-specific
lowering conventions in this document do not silently become Rust semantics.

A canonical Rust source grammar will be added while PA4/PA5 are implemented,
following CPPGM's shared grammar approach. Do not publish an incomplete grammar
as the complete language contract. The versioned Rust Reference currently owns
source syntax; each draft handout declares the subset it will implement.
