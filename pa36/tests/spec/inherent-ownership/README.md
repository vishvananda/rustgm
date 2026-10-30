# Foreign standard-library inherent ownership

`foreign-principal.rs` preserves unchanged bytes from the original PA23 local
dyn fixture proposal. Pinned Rust rejects the foreign inherent implementation.
The current source profile does not supply `std::fmt::Debug`, so an early unknown
name is only resolution-boundary evidence. Real std/sysroot-name availability
and foreign inherent ownership remain required; there is no PA23 text reference
for that earlier failure. See `contract.json` for exact origin and Rust proof.
The genuine imported user-trait graph in PA29 tests ownership independently.
