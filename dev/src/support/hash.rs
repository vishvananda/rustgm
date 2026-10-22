//! Deterministic non-cryptographic hashing for compiler-internal tables.
//!
//! Compact identities and small structural tuples dominate these tables. The
//! fast deterministic hash also handles source-controlled interned spellings;
//! it provides no collision-attack resistance or worst-case complexity bound.
//! Semantic output order must come from explicit ordered storage, never table
//! iteration. This multiplicative `FxHasher` is adapted from the CPPGM Rust
//! port (`support/fx_hash.rs`).
use std::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Fast, deterministic multiplicative hasher.
#[derive(Clone, Copy, Debug, Default)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    const fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, mut bytes: &[u8]) {
        while let Some((&chunk, rest)) = bytes.split_first_chunk::<8>() {
            self.add(u64::from_ne_bytes(chunk));
            bytes = rest;
        }
        if let Some((&chunk, rest)) = bytes.split_first_chunk::<4>() {
            self.add(u64::from(u32::from_ne_bytes(chunk)));
            bytes = rest;
        }
        if let Some((&chunk, rest)) = bytes.split_first_chunk::<2>() {
            self.add(u64::from(u16::from_ne_bytes(chunk)));
            bytes = rest;
        }
        if let Some(&byte) = bytes.first() {
            self.add(u64::from(byte));
        }
    }
    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }
    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }
    #[inline]
    fn write_u128(&mut self, i: u128) {
        #[expect(clippy::cast_possible_truncation, reason = "hash both 64-bit halves")]
        self.add(i as u64);
        self.add((i >> 64) as u64);
    }
    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }
    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.add(u64::from(i.cast_unsigned()));
    }
    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.add(u64::from(i.cast_unsigned()));
    }
    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.add(u64::from(i.cast_unsigned()));
    }
    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.add(i.cast_unsigned());
    }
    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.add(i.cast_unsigned() as u64);
    }
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// Deterministic `BuildHasher` for [`FxHasher`].
pub type BuildFxHasher = BuildHasherDefault<FxHasher>;

/// The compiler's hash map: `std::collections::HashMap` with [`FxHasher`].
pub type HashMap<K, V> = std::collections::HashMap<K, V, BuildFxHasher>;

/// The compiler's hash set: `std::collections::HashSet` with [`FxHasher`].
pub type HashSet<T> = std::collections::HashSet<T, BuildFxHasher>;
