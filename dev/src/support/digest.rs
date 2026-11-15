//! A 128-bit digest whose value depends on nothing but the bytes it reads, so
//! that every compilation on every host names equal data alike. It is
//! `SipHash-2-4` with the 128-bit output; under the fixed public key of
//! [`digest`] it guards against accidental collisions, not chosen ones.
#[cfg(test)]
#[path = "../../tests/invariants/digest_vectors.rs"]
mod tests;

const fn rounds(mut v: [u64; 4], count: u32) -> [u64; 4] {
    let mut round = 0;
    while round < count {
        v[0] = v[0].wrapping_add(v[1]);
        v[1] = v[1].rotate_left(13) ^ v[0];
        v[0] = v[0].rotate_left(32);
        v[2] = v[2].wrapping_add(v[3]);
        v[3] = v[3].rotate_left(16) ^ v[2];
        v[0] = v[0].wrapping_add(v[3]);
        v[3] = v[3].rotate_left(21) ^ v[0];
        v[2] = v[2].wrapping_add(v[1]);
        v[1] = v[1].rotate_left(17) ^ v[2];
        v[2] = v[2].rotate_left(32);
        round += 1;
    }
    v
}
const fn absorb(mut v: [u64; 4], word: u64) -> [u64; 4] {
    v[3] ^= word;
    v = rounds(v, 2);
    v[0] ^= word;
    v
}

/// `SipHash-2-4` of `bytes` under the two little-endian key words, with the
/// 128-bit output: the first output word is the low half.
#[must_use]
pub(crate) fn keyed(key: (u64, u64), bytes: &[u8]) -> u128 {
    let mut v = [
        key.0 ^ 0x736f_6d65_7073_6575,
        key.1 ^ 0x646f_7261_6e64_6f6d ^ 0xee,
        key.0 ^ 0x6c79_6765_6e65_7261,
        key.1 ^ 0x7465_6462_7974_6573,
    ];
    let mut rest = bytes;
    while let Some((&chunk, tail)) = rest.split_first_chunk::<8>() {
        v = absorb(v, u64::from_le_bytes(chunk));
        rest = tail;
    }
    // The last word holds the remaining bytes and, on top, the length modulo 256.
    let mut last = [0; 8];
    last[..rest.len()].copy_from_slice(rest);
    last[7] = bytes.len().to_le_bytes()[0];
    v = absorb(v, u64::from_le_bytes(last));
    v[2] ^= 0xee;
    v = rounds(v, 4);
    let low = v[0] ^ v[1] ^ v[2] ^ v[3];
    v[1] ^= 0xdd;
    v = rounds(v, 4);
    let high = v[0] ^ v[1] ^ v[2] ^ v[3];
    u128::from(high) << 64 | u128::from(low)
}

/// The digest of `bytes` under the zero key.
#[must_use]
pub(crate) fn digest(bytes: &[u8]) -> u128 {
    keyed((0, 0), bytes)
}
