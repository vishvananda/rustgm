//! The digest is `SipHash-2-4` with the 128-bit output, byte for byte.
use super::{digest, keyed};

/// `(length, output)` for the input bytes `0, 1, ..` under the key bytes
/// `0..=15`. The empty input is the first vector of the `SipHash` reference
/// implementation; the others were computed with `OpenSSL` 3.5 (`openssl mac
/// -macopt size:16 SIPHASH`) and cover a tail-only input, whole blocks and
/// blocks with a tail.
const VECTORS: &[(u8, u128)] = &[
    (0, 0xa381_7f04_ba25_a8e6_6df6_7214_c755_0293),
    (1, 0xda87_c1d8_6b99_af44_3476_5911_9b22_fc45),
    (7, 0xa1f1_ebbe_d8db_c153_c0b8_4aa6_1ff0_8239),
    (8, 0x3b62_a9ba_6258_f561_0f83_e264_f314_97b4),
    (9, 0x2644_9906_0ad9_baab_c47f_8b02_bb6d_71ed),
    (15, 0x5493_e999_33b0_a811_7e08_ec0f_97cf_c3d9),
    (16, 0x6ee2_a4ca_67b0_54bb_fd33_15bf_8523_0577),
    (63, 0x5150_d177_2f50_834a_503e_069a_973f_bd7c),
];

#[test]
fn outputs_are_the_published_128_bit_siphash_bytes() {
    let key = (0x0706_0504_0302_0100, 0x0f0e_0d0c_0b0a_0908);
    for &(length, output) in VECTORS {
        let input: Vec<u8> = (0..length).collect();
        // The published bytes are the two output words in little-endian order.
        assert_eq!(
            keyed(key, &input).to_le_bytes(),
            output.to_be_bytes(),
            "{length}"
        );
    }
    assert_eq!(
        digest(b"abc").to_le_bytes(),
        0x6c95_dec3_0296_2fa8_ca5e_69c1_d5d1_5478_u128.to_be_bytes()
    );
}
