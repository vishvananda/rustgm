#![no_std]
#![no_main]

use core::hint::black_box;
use core::num::NonZero;
use core::sync::atomic::{
    AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicPtr, AtomicU8, AtomicU16, AtomicU32,
    AtomicU64, AtomicUsize, Ordering,
};
use core::{mem, ptr};

#[link(name = "c")]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn abort() -> !;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { abort() }
}

/// Buffered standard output through libc `write`. Nothing here uses
/// `core::fmt`: every line is `name value` with the value in hexadecimal.
struct Out {
    buf: [u8; 256],
    len: usize,
    failed: i32,
}

impl Out {
    fn flush(&mut self) {
        let mut done = 0;
        while done < self.len {
            let n = unsafe { write(1, self.buf.as_ptr().add(done), self.len - done) };
            if n <= 0 {
                unsafe { abort() }
            }
            done += n as usize;
        }
        self.len = 0;
    }

    fn byte(&mut self, b: u8) {
        if self.len == self.buf.len() {
            self.flush();
        }
        self.buf[self.len] = b;
        self.len += 1;
    }

    fn text(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            self.byte(bytes[i]);
            i += 1;
        }
    }

    fn hex(&mut self, value: u128) {
        let mut shift = 124;
        let mut started = false;
        loop {
            let nibble = ((value >> shift) & 15) as u8;
            if nibble != 0 || started || shift == 0 {
                started = true;
                self.byte(if nibble < 10 { b'0' + nibble } else { b'a' + (nibble - 10) });
            }
            if shift == 0 {
                break;
            }
            shift -= 4;
        }
    }

    /// A value only the oracle decides.
    fn show(&mut self, name: &str, value: u128) {
        self.text(name);
        self.byte(b' ');
        self.hex(value);
        self.byte(b'\n');
    }

    /// A value this file predicts. A different value is marked on its line
    /// and counted in the exit status.
    fn check(&mut self, name: &str, got: u128, want: u128) {
        self.text(name);
        self.byte(b' ');
        self.hex(got);
        if got != want {
            self.text(" expected ");
            self.hex(want);
            self.failed += 1;
        }
        self.byte(b'\n');
    }

    /// A predicted value with a predicted flag: an overflow report, or
    /// whether an `Option` held a value.
    fn pair(&mut self, name: &str, got: (u128, bool), want: (u128, bool)) {
        self.text(name);
        self.byte(b' ');
        self.hex(got.0);
        self.byte(b' ');
        self.byte(if got.1 { b'1' } else { b'0' });
        if got.0 != want.0 || got.1 != want.1 {
            self.text(" expected ");
            self.hex(want.0);
            self.byte(b' ');
            self.byte(if want.1 { b'1' } else { b'0' });
            self.failed += 1;
        }
        self.byte(b'\n');
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut out = Out { buf: [0; 256], len: 0, failed: 0 };
    // Each section is flushed, so output stops at the section that failed.
    counts(&mut out);
    out.flush();
    nonzero(&mut out);
    out.flush();
    permutations(&mut out);
    out.flush();
    overflow(&mut out);
    out.flush();
    saturation(&mut out);
    out.flush();
    sweeps(&mut out);
    out.flush();
    division(&mut out);
    out.flush();
    pointers(&mut out);
    out.flush();
    slices(&mut out);
    out.flush();
    moves(&mut out);
    out.flush();
    atomics(&mut out);
    out.flush();
    atomic_sweeps(&mut out);
    out.flush();
    if out.failed > 100 { 100 } else { out.failed }
}

/// `count_ones`, `leading_zeros` and `trailing_zeros` at every width; zero
/// yields the width.
fn counts(out: &mut Out) {
    out.check("u8 ones 5a", black_box(0x5au8).count_ones() as u128, 4);
    out.check("u8 ones 0", black_box(0u8).count_ones() as u128, 0);
    out.check("u8 ones ff", black_box(0xffu8).count_ones() as u128, 8);
    out.check("u8 lz 0", black_box(0u8).leading_zeros() as u128, 8);
    out.check("u8 lz 1", black_box(1u8).leading_zeros() as u128, 7);
    out.check("u8 lz 10", black_box(0x10u8).leading_zeros() as u128, 3);
    out.check("u8 lz 80", black_box(0x80u8).leading_zeros() as u128, 0);
    out.check("u8 tz 0", black_box(0u8).trailing_zeros() as u128, 8);
    out.check("u8 tz 18", black_box(0x18u8).trailing_zeros() as u128, 3);
    out.check("u8 tz 80", black_box(0x80u8).trailing_zeros() as u128, 7);
    out.check("u8 leading ones f0", black_box(0xf0u8).leading_ones() as u128, 4);
    out.check("u8 leading ones ff", black_box(0xffu8).leading_ones() as u128, 8);
    out.check("u8 trailing ones 0f", black_box(0x0fu8).trailing_ones() as u128, 4);
    out.check("i8 ones -1", black_box(-1i8).count_ones() as u128, 8);
    out.check("i8 zeros -1", black_box(-1i8).count_zeros() as u128, 0);
    out.check("i8 lz min", black_box(i8::MIN).leading_zeros() as u128, 0);
    out.check("i8 lz 0", black_box(0i8).leading_zeros() as u128, 8);
    out.check("i8 lz 1", black_box(1i8).leading_zeros() as u128, 7);
    out.check("i8 tz min", black_box(i8::MIN).trailing_zeros() as u128, 7);
    out.check("u16 ones f0f0", black_box(0xf0f0u16).count_ones() as u128, 8);
    out.check("u16 lz 0", black_box(0u16).leading_zeros() as u128, 16);
    out.check("u16 lz 100", black_box(0x0100u16).leading_zeros() as u128, 7);
    out.check("u16 tz 0", black_box(0u16).trailing_zeros() as u128, 16);
    out.check("u16 tz 100", black_box(0x0100u16).trailing_zeros() as u128, 8);
    out.check("i16 ones -2", black_box(-2i16).count_ones() as u128, 15);
    out.check("i16 tz -2", black_box(-2i16).trailing_zeros() as u128, 1);
    out.check("i16 lz max", black_box(i16::MAX).leading_zeros() as u128, 1);
    out.check("u32 ones deadbeef", black_box(0xdead_beefu32).count_ones() as u128, 24);
    out.check("u32 lz 0", black_box(0u32).leading_zeros() as u128, 32);
    out.check("u32 lz 1", black_box(1u32).leading_zeros() as u128, 31);
    out.check("u32 lz 10000", black_box(0x0001_0000u32).leading_zeros() as u128, 15);
    out.check("u32 tz 0", black_box(0u32).trailing_zeros() as u128, 32);
    out.check("u32 tz 10000", black_box(0x0001_0000u32).trailing_zeros() as u128, 16);
    out.check("u32 trailing ones max", black_box(u32::MAX).trailing_ones() as u128, 32);
    out.check("i32 ones min", black_box(i32::MIN).count_ones() as u128, 1);
    out.check("i32 lz max", black_box(i32::MAX).leading_zeros() as u128, 1);
    out.check("i32 tz -1", black_box(-1i32).trailing_zeros() as u128, 0);
    out.check("u64 ones max", black_box(u64::MAX).count_ones() as u128, 64);
    out.check("u64 ones ends", black_box(0x8000_0000_0000_0001u64).count_ones() as u128, 2);
    out.check("u64 lz 0", black_box(0u64).leading_zeros() as u128, 64);
    out.check("u64 lz bit40", black_box(1u64 << 40).leading_zeros() as u128, 23);
    out.check("u64 tz 0", black_box(0u64).trailing_zeros() as u128, 64);
    out.check("u64 tz bit40", black_box(1u64 << 40).trailing_zeros() as u128, 40);
    out.check("i64 ones 0", black_box(0i64).count_ones() as u128, 0);
    out.check("i64 lz -1", black_box(-1i64).leading_zeros() as u128, 0);
    out.check("i64 tz min", black_box(i64::MIN).trailing_zeros() as u128, 63);
    out.check("u128 ones max", black_box(u128::MAX).count_ones() as u128, 128);
    out.check("u128 ones two", black_box((1u128 << 100) | 1).count_ones() as u128, 2);
    out.check("u128 lz 0", black_box(0u128).leading_zeros() as u128, 128);
    out.check("u128 lz 1", black_box(1u128).leading_zeros() as u128, 127);
    out.check("u128 lz bit100", black_box(1u128 << 100).leading_zeros() as u128, 27);
    out.check("u128 tz 0", black_box(0u128).trailing_zeros() as u128, 128);
    out.check("u128 tz bit100", black_box(1u128 << 100).trailing_zeros() as u128, 100);
    out.check("i128 ones -1", black_box(-1i128).count_ones() as u128, 128);
    out.check("i128 lz max", black_box(i128::MAX).leading_zeros() as u128, 1);
    out.check("i128 tz min", black_box(i128::MIN).trailing_zeros() as u128, 127);
    out.check("usize ones max", black_box(usize::MAX).count_ones() as u128, 64);
    out.check("usize lz 0", black_box(0usize).leading_zeros() as u128, 64);
    out.check("usize tz 4096", black_box(4096usize).trailing_zeros() as u128, 12);
    out.check("isize lz min", black_box(isize::MIN).leading_zeros() as u128, 0);
    out.check("isize tz min", black_box(isize::MIN).trailing_zeros() as u128, 63);
}

/// The forms that may assume a nonzero operand, and their users.
fn nonzero(out: &mut Out) {
    let lz8 = match NonZero::new(black_box(0x80u8)) {
        Some(n) => n.leading_zeros() * 100 + n.trailing_zeros(),
        None => 999,
    };
    out.check("nonzero u8 80", lz8 as u128, 7);
    let n32 = match NonZero::new(black_box(1u32)) {
        Some(n) => n.leading_zeros() * 100 + n.trailing_zeros(),
        None => 999,
    };
    out.check("nonzero u32 1", n32 as u128, 3100);
    let n64 = match NonZero::new(black_box(1u64 << 40)) {
        Some(n) => n.leading_zeros() * 100 + n.trailing_zeros(),
        None => 999,
    };
    out.check("nonzero u64 bit40", n64 as u128, 2340);
    let n128 = match NonZero::new(black_box(1u128 << 100)) {
        Some(n) => n.leading_zeros() * 1000 + n.trailing_zeros(),
        None => 999,
    };
    out.check("nonzero u128 bit100", n128 as u128, 27100);
    let n16 = match NonZero::new(black_box(0xfffeu16)) {
        Some(n) => n.leading_zeros() * 100 + n.trailing_zeros(),
        None => 999,
    };
    out.check("nonzero u16 fffe", n16 as u128, 1);
    out.check("nonzero none", NonZero::new(black_box(0u16)).is_none() as u128, 1);
    out.check("ilog2 1000", black_box(1000u32).ilog2() as u128, 9);
    out.check("ilog2 u8 1", black_box(1u8).ilog2() as u128, 0);
    out.check("ilog2 u64 max", black_box(u64::MAX).ilog2() as u128, 63);
    out.check("ilog2 u128 bit100", black_box(1u128 << 100).ilog2() as u128, 100);
    out.check("next power 5", black_box(5u32).next_power_of_two() as u128, 8);
    out.check("next power 1", black_box(1u32).next_power_of_two() as u128, 1);
    out.check("next power u8 65", black_box(65u8).next_power_of_two() as u128, 128);
    let none = black_box(0x8000_0001u32).checked_next_power_of_two();
    out.check("next power none", none.is_none() as u128, 1);
    out.check("power of two 64", black_box(64u64).is_power_of_two() as u128, 1);
    out.check("power of two 65", black_box(65u64).is_power_of_two() as u128, 0);
}

/// `swap_bytes` and `reverse_bits`, and the endian conversions built on them.
fn permutations(out: &mut Out) {
    out.check("u8 swap", black_box(0x12u8).swap_bytes() as u128, 0x12);
    out.check("u16 swap", black_box(0x1234u16).swap_bytes() as u128, 0x3412);
    out.check("i16 swap", black_box(0x1234i16).swap_bytes() as u16 as u128, 0x3412);
    out.check("u32 swap", black_box(0x1234_5678u32).swap_bytes() as u128, 0x7856_3412);
    out.check("i32 swap -2", black_box(-2i32).swap_bytes() as u32 as u128, 0xfeff_ffff);
    out.check(
        "u64 swap",
        black_box(0x0123_4567_89ab_cdefu64).swap_bytes() as u128,
        0xefcd_ab89_6745_2301,
    );
    out.check(
        "usize swap",
        black_box(0x0102_0304_0506_0708usize).swap_bytes() as u128,
        0x0807_0605_0403_0201,
    );
    out.check(
        "u128 swap",
        black_box(0x0011_2233_4455_6677_8899_aabb_ccdd_eeffu128).swap_bytes(),
        0xffee_ddcc_bbaa_9988_7766_5544_3322_1100,
    );
    out.check("u32 to_be", black_box(0x1234_5678u32).to_be() as u128, 0x7856_3412);
    out.check("u32 to_le", black_box(0x1234_5678u32).to_le() as u128, 0x1234_5678);
    let bytes = black_box([0x12u8, 0x34, 0x56, 0x78]);
    out.check("u32 from_be_bytes", u32::from_be_bytes(bytes) as u128, 0x1234_5678);
    out.check("u32 from_le_bytes", u32::from_le_bytes(bytes) as u128, 0x7856_3412);
    out.check("u16 to_be_bytes", black_box(0x1234u16).to_be_bytes()[0] as u128, 0x12);
    out.check("u8 reverse 01", black_box(0x01u8).reverse_bits() as u128, 0x80);
    out.check("u8 reverse 12", black_box(0x12u8).reverse_bits() as u128, 0x48);
    out.check("i8 reverse min", black_box(i8::MIN).reverse_bits() as u8 as u128, 1);
    out.check("u16 reverse", black_box(0x1234u16).reverse_bits() as u128, 0x2c48);
    out.check("u32 reverse", black_box(0x1234_5678u32).reverse_bits() as u128, 0x1e6a_2c48);
    out.check(
        "u64 reverse",
        black_box(0x0123_4567_89ab_cdefu64).reverse_bits() as u128,
        0xf7b3_d591_e6a2_c480,
    );
    out.check("u64 reverse 1", black_box(1u64).reverse_bits() as u128, 1 << 63);
    out.check("u128 reverse 1", black_box(1u128).reverse_bits(), 1 << 127);
    out.check("u128 reverse top", black_box(3u128 << 126).reverse_bits(), 3);
}

/// `overflowing_add` and `checked_add`: the wrapped sum and whether it overflowed.
fn overflow(out: &mut Out) {
    let (v, f) = black_box(200u8).overflowing_add(100);
    out.pair("u8 200+100", (v as u128, f), (44, true));
    let (v, f) = black_box(100i8).overflowing_add(100);
    out.pair("i8 100+100", (v as u8 as u128, f), (0xc8, true));
    let (v, f) = black_box(-100i8).overflowing_add(-100);
    out.pair("i8 -100-100", (v as u8 as u128, f), (56, true));
    let (v, f) = black_box(100i8).overflowing_add(27);
    out.pair("i8 100+27", (v as u8 as u128, f), (127, false));
    let (v, f) = black_box(0xffffu16).overflowing_add(1);
    out.pair("u16 max+1", (v as u128, f), (0, true));
    let (v, f) = black_box(0xfffeu16).overflowing_add(1);
    out.pair("u16 max-1+1", (v as u128, f), (0xffff, false));
    let (v, f) = black_box(i16::MAX).overflowing_add(1);
    out.pair("i16 max+1", (v as u16 as u128, f), (0x8000, true));
    let (v, f) = black_box(i16::MIN).overflowing_add(-1);
    out.pair("i16 min-1", (v as u16 as u128, f), (0x7fff, true));
    let (v, f) = black_box(u32::MAX).overflowing_add(u32::MAX);
    out.pair("u32 max+max", (v as u128, f), (0xffff_fffe, true));
    let (v, f) = black_box(i32::MIN).overflowing_add(i32::MIN);
    out.pair("i32 min+min", (v as u32 as u128, f), (0, true));
    let (v, f) = black_box(i32::MAX).overflowing_add(i32::MIN);
    out.pair("i32 max+min", (v as u32 as u128, f), (0xffff_ffff, false));
    let (v, f) = black_box(u64::MAX).overflowing_add(1);
    out.pair("u64 max+1", (v as u128, f), (0, true));
    let (v, f) = black_box(i64::MAX).overflowing_add(1);
    out.pair("i64 max+1", (v as u64 as u128, f), (1 << 63, true));
    let (v, f) = black_box(-5i64).overflowing_add(3);
    out.pair("i64 -5+3", (v as u64 as u128, f), (0xffff_ffff_ffff_fffe, false));
    let (v, f) = black_box(usize::MAX).overflowing_add(2);
    out.pair("usize max+2", (v as u128, f), (1, true));
    let (v, f) = black_box(isize::MIN).overflowing_add(isize::MIN);
    out.pair("isize min+min", (v as usize as u128, f), (0, true));
    let (v, f) = black_box(u128::MAX).overflowing_add(1);
    out.pair("u128 max+1", (v, f), (0, true));
    let (v, f) = black_box(1u128 << 127).overflowing_add(1 << 127);
    out.pair("u128 top+top", (v, f), (0, true));
    let (v, f) = black_box(1u128 << 64).overflowing_add(1 << 64);
    out.pair("u128 carry", (v, f), (1 << 65, false));
    let (v, f) = black_box(i128::MAX).overflowing_add(1);
    out.pair("i128 max+1", (v as u128, f), (1 << 127, true));
    let (v, f) = black_box(i128::MIN).overflowing_add(-1);
    out.pair("i128 min-1", (v as u128, f), (u128::MAX >> 1, true));
    let (v, f) = black_box(-1i128).overflowing_add(1);
    out.pair("i128 -1+1", (v as u128, f), (0, false));
    let sum = match black_box(200u8).checked_add(55) {
        Some(v) => (v as u128, true),
        None => (0, false),
    };
    out.pair("u8 checked 200+55", sum, (255, true));
    let sum = match black_box(200u8).checked_add(56) {
        Some(v) => (v as u128, true),
        None => (0, false),
    };
    out.pair("u8 checked 200+56", sum, (0, false));
    let sum = match black_box(i8::MIN).checked_add(-1) {
        Some(v) => (v as u8 as u128, true),
        None => (0, false),
    };
    out.pair("i8 checked min-1", sum, (0, false));
    let sum = match black_box(i64::MAX).checked_add(0) {
        Some(v) => (v as u64 as u128, true),
        None => (0, false),
    };
    out.pair("i64 checked max+0", sum, (u64::MAX as u128 >> 1, true));
    let sum = match black_box(u128::MAX - 1).checked_add(1) {
        Some(v) => (v, true),
        None => (0, false),
    };
    out.pair("u128 checked max-1+1", sum, (u128::MAX, true));
    let sum = match black_box(i128::MIN).checked_add(i128::MIN) {
        Some(v) => (v as u128, true),
        None => (0, false),
    };
    out.pair("i128 checked min+min", sum, (0, false));
}

/// `saturating_add` and `saturating_sub`: an overflowing signed result takes
/// the bound on the side of the left operand's sign.
fn saturation(out: &mut Out) {
    out.check("u8 200+100", black_box(200u8).saturating_add(100) as u128, 255);
    out.check("u8 200+55", black_box(200u8).saturating_add(55) as u128, 255);
    out.check("u8 200+54", black_box(200u8).saturating_add(54) as u128, 254);
    out.check("u8 5-10", black_box(5u8).saturating_sub(10) as u128, 0);
    out.check("u8 10-5", black_box(10u8).saturating_sub(5) as u128, 5);
    out.check("i8 100+100", black_box(100i8).saturating_add(100) as u8 as u128, 127);
    out.check("i8 -100-100", black_box(-100i8).saturating_add(-100) as u8 as u128, 0x80);
    out.check("i8 -100+100", black_box(-100i8).saturating_add(100) as u8 as u128, 0);
    out.check("i8 100--100", black_box(100i8).saturating_sub(-100) as u8 as u128, 127);
    out.check("i8 -100-100 sub", black_box(-100i8).saturating_sub(100) as u8 as u128, 0x80);
    out.check("i8 0-min", black_box(0i8).saturating_sub(i8::MIN) as u8 as u128, 127);
    out.check("i8 -1-min", black_box(-1i8).saturating_sub(i8::MIN) as u8 as u128, 127);
    out.check("i8 -2-min", black_box(-2i8).saturating_sub(i8::MIN) as u8 as u128, 126);
    out.check("u16 max+1", black_box(u16::MAX).saturating_add(1) as u128, 0xffff);
    out.check("u16 0-1", black_box(0u16).saturating_sub(1) as u128, 0);
    out.check("i16 max+1", black_box(i16::MAX).saturating_add(1) as u16 as u128, 0x7fff);
    out.check("i16 min-1", black_box(i16::MIN).saturating_sub(1) as u16 as u128, 0x8000);
    out.check("u32 max+max", black_box(u32::MAX).saturating_add(u32::MAX) as u128, 0xffff_ffff);
    out.check("u32 7-max", black_box(7u32).saturating_sub(u32::MAX) as u128, 0);
    out.check("i32 min+-1", black_box(i32::MIN).saturating_add(-1) as u32 as u128, 1 << 31);
    out.check("i32 max-min", black_box(i32::MAX).saturating_sub(i32::MIN) as u32 as u128, 0x7fff_ffff);
    out.check("i32 3-5", black_box(3i32).saturating_sub(5) as u32 as u128, 0xffff_fffe);
    out.check("u64 max+1", black_box(u64::MAX).saturating_add(1) as u128, u64::MAX as u128);
    out.check("u64 1-2", black_box(1u64).saturating_sub(2) as u128, 0);
    out.check("i64 max+max", black_box(i64::MAX).saturating_add(i64::MAX) as u64 as u128, u64::MAX as u128 >> 1);
    out.check("i64 min-max", black_box(i64::MIN).saturating_sub(i64::MAX) as u64 as u128, 1 << 63);
    out.check("usize max+1", black_box(usize::MAX).saturating_add(1) as u128, u64::MAX as u128);
    out.check("usize 0-1", black_box(0usize).saturating_sub(1) as u128, 0);
    out.check("isize min-1", black_box(isize::MIN).saturating_sub(1) as usize as u128, 1 << 63);
    out.check("isize max+1", black_box(isize::MAX).saturating_add(1) as usize as u128, u64::MAX as u128 >> 1);
    out.check("u128 max+max", black_box(u128::MAX).saturating_add(u128::MAX), u128::MAX);
    out.check("u128 max-1+1", black_box(u128::MAX - 1).saturating_add(1), u128::MAX);
    out.check("u128 0-1", black_box(0u128).saturating_sub(1), 0);
    out.check("u128 big-small", black_box(1u128 << 100).saturating_sub(1), (1 << 100) - 1);
    out.check("i128 max+max", black_box(i128::MAX).saturating_add(i128::MAX) as u128, u128::MAX >> 1);
    out.check("i128 min+min", black_box(i128::MIN).saturating_add(i128::MIN) as u128, 1 << 127);
    out.check("i128 min-max", black_box(i128::MIN).saturating_sub(i128::MAX) as u128, 1 << 127);
    out.check("i128 max-min", black_box(i128::MAX).saturating_sub(i128::MIN) as u128, u128::MAX >> 1);
    out.check("i128 -1+1", black_box(-1i128).saturating_add(1) as u128, 0);
}

fn next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (*state >> 24) ^ (*state << 40)
}

fn naive_ones(mut value: u128) -> u32 {
    let mut count = 0;
    while value != 0 {
        count += (value & 1) as u32;
        value >>= 1;
    }
    count
}

fn naive_leading(value: u128, bits: u32) -> u32 {
    let mut count = 0;
    while count < bits && (value >> (bits - 1 - count)) & 1 == 0 {
        count += 1;
    }
    count
}

fn naive_trailing(value: u128, bits: u32) -> u32 {
    let mut count = 0;
    while count < bits && (value >> count) & 1 == 0 {
        count += 1;
    }
    count
}

fn naive_reverse(value: u128, bits: u32) -> u128 {
    let mut result = 0;
    let mut bit = 0;
    while bit < bits {
        result = (result << 1) | ((value >> bit) & 1);
        bit += 1;
    }
    result
}

fn naive_swap(value: u128, bits: u32) -> u128 {
    let mut result = 0;
    let mut bit = 0;
    while bit < bits {
        result = (result << 8) | ((value >> bit) & 0xff);
        bit += 8;
    }
    result
}

/// Samples near both bounds, near zero and across the whole range.
macro_rules! sample {
    ($state:ident, $t:ty) => {{
        let r = next(&mut $state);
        let small = ((r >> 3) & 3) as $t;
        match r & 7 {
            0 => <$t>::MAX.wrapping_sub(small),
            1 => <$t>::MIN.wrapping_add(small),
            2 => small,
            3 => (0 as $t).wrapping_sub(small),
            _ => (next(&mut $state) as u128 | (r as u128) << 64) as $t,
        }
    }};
}

/// Compares the bit operations of one type with bit-by-bit loops. `bad`
/// counts disagreements; `sum` is a checksum of every result.
macro_rules! bit_sweep {
    ($out:ident, $bad:literal, $sum:literal, $t:ty, $u:ty) => {{
        let mut state = 0x9e37_79b9_7f4a_7c15u64;
        let mut sum = 0u64;
        let mut bad = 0u32;
        let mut k = 0;
        while k < 200 {
            let v = sample!(state, $t);
            let wide = v as $u as u128;
            let bits = <$t>::BITS;
            let ones = v.count_ones();
            let leading = v.leading_zeros();
            let trailing = v.trailing_zeros();
            let reversed = v.reverse_bits() as $u as u128;
            let swapped = v.swap_bytes() as $u as u128;
            bad += (ones != naive_ones(wide)) as u32
                + (leading != naive_leading(wide, bits)) as u32
                + (trailing != naive_trailing(wide, bits)) as u32
                + (reversed != naive_reverse(wide, bits)) as u32
                + (swapped != naive_swap(wide, bits)) as u32;
            sum = sum
                .wrapping_mul(1_000_003)
                .wrapping_add((ones + leading * 131 + trailing * 17161) as u64)
                .wrapping_add((reversed ^ (reversed >> 64)) as u64)
                .wrapping_add(((swapped ^ (swapped >> 64)) as u64) << 1);
            k += 1;
        }
        $out.check($bad, bad as u128, 0);
        $out.show($sum, sum as u128);
    }};
}

/// Compares overflowing, checked and saturating sums and differences of one
/// type of at most 64 bits with 128-bit arithmetic.
macro_rules! sum_sweep {
    ($out:ident, $bad:literal, $sum:literal, $t:ty, $u:ty) => {{
        let mut state = 0x2545_f491_4f6c_dd1du64;
        let mut sum = 0u64;
        let mut bad = 0u32;
        let (low, high) = (<$t>::MIN as i128, <$t>::MAX as i128);
        let mut k = 0;
        while k < 2000 {
            let a = sample!(state, $t);
            let b = sample!(state, $t);
            let add = a as i128 + b as i128;
            let sub = a as i128 - b as i128;
            let (wrapped, flag) = a.overflowing_add(b);
            let checked = match a.checked_add(b) {
                Some(v) => !flag && v == wrapped,
                None => flag,
            };
            let up = a.saturating_add(b);
            let down = a.saturating_sub(b);
            bad += (wrapped != add as $t) as u32
                + (flag != (add < low || add > high)) as u32
                + !checked as u32
                + (up as i128 != if add > high { high } else if add < low { low } else { add }) as u32
                + (down as i128 != if sub > high { high } else if sub < low { low } else { sub }) as u32;
            sum = sum
                .wrapping_mul(1_000_003)
                .wrapping_add(wrapped as $u as u64)
                .wrapping_add((up as $u as u64) << 1)
                .wrapping_add((down as $u as u64) << 2)
                .wrapping_add(flag as u64);
            k += 1;
        }
        $out.check($bad, bad as u128, 0);
        $out.show($sum, sum as u128);
    }};
}

fn sweeps(out: &mut Out) {
    bit_sweep!(out, "bits u8 bad", "bits u8 sum", u8, u8);
    bit_sweep!(out, "bits i8 bad", "bits i8 sum", i8, u8);
    bit_sweep!(out, "bits u16 bad", "bits u16 sum", u16, u16);
    bit_sweep!(out, "bits i16 bad", "bits i16 sum", i16, u16);
    bit_sweep!(out, "bits u32 bad", "bits u32 sum", u32, u32);
    bit_sweep!(out, "bits i32 bad", "bits i32 sum", i32, u32);
    bit_sweep!(out, "bits u64 bad", "bits u64 sum", u64, u64);
    bit_sweep!(out, "bits i64 bad", "bits i64 sum", i64, u64);
    bit_sweep!(out, "bits usize bad", "bits usize sum", usize, usize);
    bit_sweep!(out, "bits isize bad", "bits isize sum", isize, usize);
    bit_sweep!(out, "bits u128 bad", "bits u128 sum", u128, u128);
    bit_sweep!(out, "bits i128 bad", "bits i128 sum", i128, u128);
    sum_sweep!(out, "sums u8 bad", "sums u8 sum", u8, u8);
    sum_sweep!(out, "sums i8 bad", "sums i8 sum", i8, u8);
    sum_sweep!(out, "sums u16 bad", "sums u16 sum", u16, u16);
    sum_sweep!(out, "sums i16 bad", "sums i16 sum", i16, u16);
    sum_sweep!(out, "sums u32 bad", "sums u32 sum", u32, u32);
    sum_sweep!(out, "sums i32 bad", "sums i32 sum", i32, u32);
    sum_sweep!(out, "sums u64 bad", "sums u64 sum", u64, u64);
    sum_sweep!(out, "sums i64 bad", "sums i64 sum", i64, u64);
    sum_sweep!(out, "sums usize bad", "sums usize sum", usize, usize);
    sum_sweep!(out, "sums isize bad", "sums isize sum", isize, usize);
}

/// Exact division: whole chunks of a slice, and the element count that
/// aligns a pointer.
fn division(out: &mut Out) {
    let words = [1u32, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let (chunks, rest) = black_box(&words[..]).as_chunks::<4>();
    out.check("chunks of 4", chunks.len() as u128, 3);
    out.check("chunks of 4 rest", rest.len() as u128, 0);
    out.check("chunk element", chunks[2][1] as u128, 10);
    let bytes = [7u8; 10];
    let (chunks, rest) = black_box(&bytes[..]).as_chunks::<3>();
    out.check("chunks of 3", chunks.len() as u128, 3);
    out.check("chunks of 3 rest", rest.len() as u128, 1);
    let halves = [0u16; 16];
    let p = black_box(halves.as_ptr());
    let count = p.align_offset(8);
    let aligned = count < 4 && (p as usize + 2 * count) % 8 == 0;
    out.check("align u16 to 8", aligned as u128, 1);
    let triples = [[0u8; 3]; 8];
    let p = black_box(triples.as_ptr());
    let count = p.align_offset(4);
    let aligned = count < 4 && (p as usize + 3 * count) % 4 == 0;
    out.check("align triple to 4", aligned as u128, 1);
    let p = black_box(bytes.as_ptr());
    let count = p.align_offset(16);
    let aligned = count < 16 && (p as usize + count) % 16 == 0;
    out.check("align u8 to 16", aligned as u128, 1);
}

/// `add`, `sub`, `offset` and the two `offset_from` forms scale by the
/// pointee size; a zero-sized pointee leaves the address unchanged.
fn pointers(out: &mut Out) {
    let squares = [1u32, 2, 5, 10, 17, 26, 37, 50];
    let p = black_box(squares.as_ptr());
    unsafe {
        out.check("u32 add 3", *p.add(3) as u128, 10);
        out.check("u32 add 7 sub 2", *p.add(7).sub(2) as u128, 26);
        out.check("u32 offset -3", *p.add(4).offset(-3) as u128, 2);
        out.check("u32 offset 0", *p.offset(0) as u128, 1);
        out.check("u32 distance", p.add(6).offset_from(p) as usize as u128, 6);
        out.check("u32 distance back", p.offset_from(p.add(6)) as usize as u128, 0xffff_ffff_ffff_fffa);
        out.check("u32 unsigned distance", p.add(6).offset_from_unsigned(p) as u128, 6);
        out.check("u32 byte distance", (p.add(6) as usize - p as usize) as u128, 24);
    }
    let triples = [[1u8, 2, 3], [4, 5, 6], [7, 8, 9], [10, 11, 12], [13, 14, 15]];
    let t = black_box(triples.as_ptr());
    unsafe {
        out.check("triple add 3", (*t.add(3))[1] as u128, 11);
        out.check("triple distance", t.add(4).offset_from(t) as usize as u128, 4);
        out.check("triple distance back", t.add(1).offset_from(t.add(4)) as usize as u128, 0xffff_ffff_ffff_fffd);
        out.check("triple unsigned distance", t.add(4).offset_from_unsigned(t.add(1)) as u128, 3);
        out.check("triple byte distance", (t.add(5) as usize - t as usize) as u128, 15);
    }
    let bytes = *b"intrinsic";
    let b = black_box(bytes.as_ptr());
    unsafe {
        out.check("u8 add 8", *b.add(8) as u128, 0x63);
        out.check("u8 distance", b.add(9).offset_from(b) as usize as u128, 9);
        out.check("u8 unsigned distance", b.add(9).offset_from_unsigned(b.add(2)) as u128, 7);
    }
    let wide = [1u128 << 100, 7, u128::MAX];
    let w = black_box(wide.as_ptr());
    unsafe {
        out.check("u128 add 2", *w.add(2), u128::MAX);
        out.check("u128 add 2 sub 1", *w.add(2).sub(1), 7);
        out.check("u128 distance", w.add(3).offset_from(w) as usize as u128, 3);
    }
    let unit = ();
    let z = black_box(&unit as *const ());
    out.check("unit add 5", (unsafe { z.add(5) } == z) as u128, 1);
    let mut cells = [0u64; 4];
    let m = black_box(cells.as_mut_ptr());
    unsafe {
        *m.add(2) = 9;
        *m.add(3).sub(3) = 4;
        *m.add(1).offset(2) = 6;
        out.check("mut distance", m.add(3).offset_from(m) as usize as u128, 3);
    }
    out.check("mut writes", (cells[0] * 100 + cells[2] * 10 + cells[3]) as u128, 496);
}

/// `get_unchecked` and the checked accessors built on it.
fn slices(out: &mut Out) {
    let mut data = [3u16, 1, 4, 1, 5, 9, 2, 6];
    let s = black_box(&mut data[..]);
    unsafe {
        out.check("get_unchecked 5", *s.get_unchecked(5) as u128, 9);
        *s.get_unchecked_mut(1) = 7;
    }
    out.check("get 1", match s.get(1) { Some(v) => *v as u128, None => 99 }, 7);
    out.check("get 8", match s.get(8) { Some(v) => *v as u128, None => 99 }, 99);
    if let Some(v) = s.get_mut(7) {
        *v += 10;
    }
    out.check("get_mut 7", s[7] as u128, 16);
    let middle = &s[2..5];
    out.check("range", (middle.len() as u16 * 100 + middle[0] * 10 + middle[2]) as u128, 345);
    let triples = [[1u8, 2, 3], [4, 5, 6], [7, 8, 9]];
    let t = black_box(&triples[..]);
    out.check("triple get_unchecked", unsafe { t.get_unchecked(2) }[1] as u128, 8);
    out.check("triple get none", t.get(3).is_none() as u128, 1);
    let units = [(); 6];
    let u = black_box(&units[..]);
    out.check("unit get 5", u.get(5).is_some() as u128, 1);
    out.check("unit get 6", u.get(6).is_some() as u128, 0);
    let text = black_box("intrinsic").as_bytes();
    out.check("bytes get_unchecked", unsafe { *text.get_unchecked(2) } as u128, 0x74);
    let wide = [u128::MAX, 5, 1 << 90];
    let w = black_box(&wide[..]);
    out.check("u128 get 2", match w.get(2) { Some(v) => *v, None => 0 }, 1 << 90);
}

struct Record {
    a: u64,
    b: u16,
    c: [u8; 5],
}

/// `ptr::read`, `ptr::write`, `mem::replace` and `black_box` move values of
/// every representation: scalars, aggregates, wide pointers and nothing.
fn moves(out: &mut Out) {
    let x = black_box(0x1234_5678_9abc_def0u64);
    out.check("read u64", unsafe { ptr::read(&x) } as u128, 0x1234_5678_9abc_def0);
    let x = black_box(0xa5u8);
    out.check("read u8", unsafe { ptr::read(&x) } as u128, 0xa5);
    let x = black_box(1u128 << 99);
    out.check("read u128", unsafe { ptr::read(&x) }, 1 << 99);
    let x = black_box(true);
    out.check("read bool", unsafe { ptr::read(&x) } as u128, 1);
    let x = black_box('\u{3bb}');
    out.check("read char", unsafe { ptr::read(&x) } as u128, 0x3bb);
    let x = black_box(1.5f64);
    out.check("read f64", unsafe { ptr::read(&x) }.to_bits() as u128, 0x3ff8_0000_0000_0000);
    let record = Record { a: 0x1122_3344_5566_7788, b: 0x99aa, c: [1, 2, 3, 4, 5] };
    let copy = unsafe { ptr::read(black_box(&record)) };
    out.check("read record a", copy.a as u128, 0x1122_3344_5566_7788);
    out.check("read record b", copy.b as u128, 0x99aa);
    out.check("read record c", (copy.c[0] * 10 + copy.c[4]) as u128, 15);
    let pair = black_box((7u8, 0x0102_0304u32));
    let copy = unsafe { ptr::read(&pair) };
    out.check("read pair", (copy.0 as u32 + copy.1) as u128, 0x0102_030b);
    let array = black_box([10u16, 20, 30]);
    let copy = unsafe { (&array as *const [u16; 3]).read() };
    out.check("read array", (copy[0] + copy[1] + copy[2]) as u128, 60);
    let words: &[u16] = black_box(&array[1..]);
    let copy = unsafe { ptr::read(&words) };
    out.check("read slice reference", (copy.len() as u16 * 100 + copy[1]) as u128, 230);
    let some: Option<&u16> = black_box(Some(&array[2]));
    let copy = unsafe { ptr::read(&some) };
    out.check("read option", match copy { Some(v) => *v as u128, None => 0 }, 30);
    let unit = ();
    unsafe { ptr::read(black_box(&unit)) };

    let mut slot = 0u32;
    unsafe { ptr::write(black_box(&mut slot), 0xdead_beef) };
    out.check("write u32", slot as u128, 0xdead_beef);
    let mut slot = 0u128;
    unsafe { ptr::write(black_box(&mut slot), 1 << 99) };
    out.check("write u128", slot, 1 << 99);
    let mut slot = false;
    unsafe { (black_box(&mut slot) as *mut bool).write(true) };
    out.check("write bool", slot as u128, 1);
    let mut slot = Record { a: 0, b: 0, c: [0; 5] };
    unsafe { ptr::write(black_box(&mut slot), Record { a: 1, b: 2, c: [9; 5] }) };
    out.check("write record", (slot.a * 100 + slot.b as u64 * 10 + slot.c[3] as u64) as u128, 129);
    let mut slot = (0u8, 0u64);
    unsafe { ptr::write(black_box(&mut slot), (7, 8)) };
    out.check("write pair", (slot.0 as u64 * 10 + slot.1) as u128, 78);
    let mut slot: &[u16] = &[];
    unsafe { ptr::write(black_box(&mut slot), &array[..2]) };
    out.check("write slice reference", (slot.len() as u16 * 100 + slot[1]) as u128, 220);
    let mut slot = ();
    unsafe { ptr::write(black_box(&mut slot), ()) };

    let mut current = black_box(5u32);
    let old = mem::replace(&mut current, 6);
    out.check("replace u32", (old * 10 + current) as u128, 56);
    let mut current = Record { a: 3, b: 4, c: [5; 5] };
    let old = mem::replace(black_box(&mut current), Record { a: 6, b: 7, c: [8; 5] });
    out.check("replace record old", (old.a * 100 + old.b as u64 * 10 + old.c[4] as u64) as u128, 345);
    out.check("replace record new", (current.a * 100 + current.b as u64 * 10 + current.c[0] as u64) as u128, 678);

    out.check("black_box u32", black_box(7u32) as u128, 7);
    let pair = black_box((1u8, 2u64));
    out.check("black_box pair", (pair.0 as u64 + pair.1) as u128, 3);
    let record = black_box(Record { a: 9, b: 8, c: [7; 5] });
    out.check("black_box record", (record.a * 100 + record.b as u64 * 10 + record.c[2] as u64) as u128, 987);
    let present: Option<u32> = black_box(Some(41));
    let value = match present {
        Some(v) => v + 1,
        None => unsafe { core::hint::unreachable_unchecked() },
    };
    out.check("unreachable arm not taken", value as u128, 42);
}

static COUNTER: AtomicUsize = AtomicUsize::new(5);
static FLAG: AtomicBool = AtomicBool::new(false);

/// A compare-exchange result as the value it carries and whether it stored.
macro_rules! outcome {
    ($result:expr) => {
        match $result {
            Ok(seen) => (seen as u128, true),
            Err(seen) => (seen as u128, false),
        }
    };
}

/// Atomic integers, booleans and pointers. A fetch operation returns the
/// value before it; unsigned and signed extrema differ at the top bit.
fn atomics(out: &mut Out) {
    let small = AtomicU8::new(200);
    out.check("u8 load", small.load(Ordering::SeqCst) as u128, 200);
    small.store(250, Ordering::Release);
    out.check("u8 swap", small.swap(7, Ordering::AcqRel) as u128, 250);
    out.check("u8 add", small.fetch_add(250, Ordering::Relaxed) as u128, 7);
    out.check("u8 sub", small.fetch_sub(3, Ordering::Relaxed) as u128, 1);
    out.check("u8 and", small.fetch_and(0x3c, Ordering::Acquire) as u128, 0xfe);
    out.check("u8 nand", small.fetch_nand(0x0f, Ordering::Relaxed) as u128, 0x3c);
    out.check("u8 or", small.fetch_or(0x0c, Ordering::Relaxed) as u128, 0xf3);
    out.check("u8 xor", small.fetch_xor(0xf0, Ordering::Relaxed) as u128, 0xff);
    out.check("u8 max", small.fetch_max(0x80, Ordering::Relaxed) as u128, 0x0f);
    out.check("u8 min", small.fetch_min(0x7f, Ordering::Relaxed) as u128, 0x80);
    out.check("u8 max kept", small.fetch_max(3, Ordering::Relaxed) as u128, 0x7f);
    let exchanged = small.compare_exchange(0x7f, 9, Ordering::SeqCst, Ordering::SeqCst);
    out.pair("u8 exchange", outcome!(exchanged), (0x7f, true));
    let exchanged = small.compare_exchange(0x7f, 10, Ordering::AcqRel, Ordering::Relaxed);
    out.pair("u8 exchange stale", outcome!(exchanged), (9, false));
    out.check("u8 final", small.load(Ordering::Relaxed) as u128, 9);

    let signed = AtomicI8::new(1);
    out.check("i8 max kept", signed.fetch_max(-1, Ordering::Relaxed) as u8 as u128, 1);
    out.check("i8 min", signed.fetch_min(-1, Ordering::Relaxed) as u8 as u128, 1);
    out.check("i8 min kept", signed.fetch_min(127, Ordering::Relaxed) as u8 as u128, 0xff);
    out.check("i8 add", signed.fetch_add(-128, Ordering::Relaxed) as u8 as u128, 0xff);
    out.check("i8 sub", signed.fetch_sub(-1, Ordering::Relaxed) as u8 as u128, 0x7f);
    out.check("i8 load", signed.load(Ordering::Relaxed) as u8 as u128, 0x80);

    let word = AtomicU16::new(0xfff0);
    out.check("u16 add", word.fetch_add(0x20, Ordering::Release) as u128, 0xfff0);
    out.check("u16 max", word.fetch_max(0x8000, Ordering::Relaxed) as u128, 0x10);
    out.check("u16 load", word.load(Ordering::Acquire) as u128, 0x8000);
    let short = AtomicI16::new(-2);
    out.check("i16 or", short.fetch_or(1, Ordering::SeqCst) as u16 as u128, 0xfffe);
    out.check("i16 swap", short.swap(300, Ordering::SeqCst) as u16 as u128, 0xffff);
    out.check("i16 max kept", short.fetch_max(-300, Ordering::SeqCst) as u16 as u128, 300);
    out.check("i16 min", short.fetch_min(-300, Ordering::SeqCst) as u16 as u128, 300);
    out.check("i16 load", short.load(Ordering::SeqCst) as u16 as u128, 0xfed4);

    let wide = AtomicU32::new(0xffff_fff0);
    out.check("u32 add", wide.fetch_add(0x20, Ordering::Release) as u128, 0xffff_fff0);
    out.check("u32 sub", wide.fetch_sub(0x11, Ordering::Release) as u128, 0x10);
    out.check("u32 min", wide.fetch_min(5, Ordering::Relaxed) as u128, 0xffff_ffff);
    out.check("u32 load", wide.load(Ordering::Acquire) as u128, 5);
    let narrow = AtomicI32::new(i32::MIN);
    out.check("i32 sub", narrow.fetch_sub(1, Ordering::AcqRel) as u32 as u128, 0x8000_0000);
    out.check("i32 xor", narrow.fetch_xor(-1, Ordering::AcqRel) as u32 as u128, 0x7fff_ffff);
    out.check("i32 max", narrow.fetch_max(0, Ordering::AcqRel) as u32 as u128, 0x8000_0000);
    out.check("i32 load", narrow.load(Ordering::Acquire) as u32 as u128, 0);

    let big = AtomicU64::new(u64::MAX - 1);
    out.check("u64 add", big.fetch_add(3, Ordering::SeqCst) as u128, 0xffff_ffff_ffff_fffe);
    out.check("u64 max", big.fetch_max(1 << 63, Ordering::SeqCst) as u128, 1);
    out.check("u64 min kept", big.fetch_min(u64::MAX, Ordering::SeqCst) as u128, 1 << 63);
    out.check("u64 swap", big.swap(1 << 40, Ordering::SeqCst) as u128, 1 << 63);
    let exchanged = big.compare_exchange(1 << 40, 9, Ordering::Release, Ordering::Relaxed);
    out.pair("u64 exchange", outcome!(exchanged), (1 << 40, true));
    let total = AtomicI64::new(-5);
    out.check("i64 sub", total.fetch_sub(10, Ordering::SeqCst) as u64 as u128, 0xffff_ffff_ffff_fffb);
    out.check("i64 max", total.fetch_max(3, Ordering::Relaxed) as u64 as u128, 0xffff_ffff_ffff_fff1);
    out.check("i64 min", total.fetch_min(i64::MIN, Ordering::Relaxed) as u64 as u128, 3);
    out.check("i64 load", total.load(Ordering::Relaxed) as u64 as u128, 1 << 63);
    let size = AtomicIsize::new(-1);
    out.check("isize add", size.fetch_add(isize::MAX, Ordering::Relaxed) as usize as u128, u64::MAX as u128);
    out.check("isize and", size.fetch_and(0xff, Ordering::Relaxed) as usize as u128, 0x7fff_ffff_ffff_fffe);
    out.check("isize load", size.load(Ordering::Relaxed) as usize as u128, 0xfe);

    out.check("static add", COUNTER.fetch_add(3, Ordering::Relaxed) as u128, 5);
    let mut current = COUNTER.load(Ordering::Relaxed);
    loop {
        match COUNTER.compare_exchange_weak(current, current * 7, Ordering::SeqCst, Ordering::Relaxed) {
            Ok(_) => break,
            Err(seen) => current = seen,
        }
    }
    core::sync::atomic::fence(Ordering::SeqCst);
    out.check("static weak exchange", COUNTER.load(Ordering::SeqCst) as u128, 56);

    out.check("bool swap", FLAG.swap(true, Ordering::AcqRel) as u128, 0);
    out.check("bool and", FLAG.fetch_and(false, Ordering::SeqCst) as u128, 1);
    out.check("bool or", FLAG.fetch_or(true, Ordering::SeqCst) as u128, 0);
    out.check("bool xor", FLAG.fetch_xor(true, Ordering::SeqCst) as u128, 1);
    out.check("bool nand", FLAG.fetch_nand(true, Ordering::SeqCst) as u128, 0);
    out.check("bool not", FLAG.fetch_not(Ordering::SeqCst) as u128, 1);
    let exchanged = FLAG.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed);
    out.pair("bool exchange", outcome!(exchanged), (0, true));
    let exchanged = FLAG.compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed);
    out.pair("bool exchange stale", outcome!(exchanged), (1, false));
    FLAG.store(false, Ordering::Release);
    out.check("bool load", FLAG.load(Ordering::Acquire) as u128, 0);

    let mut first = 11u32;
    let mut second = 22u32;
    let one: *mut u32 = &mut first;
    let two: *mut u32 = &mut second;
    let slot = AtomicPtr::new(ptr::null_mut::<u32>());
    out.check("ptr null", slot.load(Ordering::Acquire).is_null() as u128, 1);
    slot.store(one, Ordering::Release);
    out.check("ptr load", unsafe { *slot.load(Ordering::SeqCst) } as u128, 11);
    out.check("ptr swap", (slot.swap(two, Ordering::AcqRel) == one) as u128, 1);
    let exchanged = match slot.compare_exchange(one, one, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(seen) => ((seen == two) as u128, true),
        Err(seen) => ((seen == two) as u128, false),
    };
    out.pair("ptr exchange stale", exchanged, (1, false));
    let exchanged = match slot.compare_exchange(two, one, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(seen) => ((seen == two) as u128, true),
        Err(seen) => ((seen == two) as u128, false),
    };
    out.pair("ptr exchange", exchanged, (1, true));
    out.check("ptr final", unsafe { *slot.load(Ordering::Relaxed) } as u128, 11);
}

/// Drives one atomic type with sampled operations beside a plain model of
/// the same type. `bad` counts returned or stored values that differ from
/// the model; `sum` is a checksum of every returned value.
macro_rules! atomic_sweep {
    ($out:ident, $bad:literal, $sum:literal, $atomic:ty, $t:ty, $u:ty) => {{
        let mut state = 0x0123_4567_89ab_cdefu64;
        let mut model = sample!(state, $t);
        let cell = <$atomic>::new(model);
        let mut sum = 0u64;
        let mut bad = 0u32;
        let mut k = 0;
        while k < 600 {
            let operand = sample!(state, $t);
            let before = model;
            let returned = match next(&mut state) % 12 {
                0 => cell.load(Ordering::SeqCst),
                1 => {
                    model = operand;
                    cell.swap(operand, Ordering::AcqRel)
                }
                2 => {
                    model = model.wrapping_add(operand);
                    cell.fetch_add(operand, Ordering::Relaxed)
                }
                3 => {
                    model = model.wrapping_sub(operand);
                    cell.fetch_sub(operand, Ordering::Release)
                }
                4 => {
                    model &= operand;
                    cell.fetch_and(operand, Ordering::Acquire)
                }
                5 => {
                    model = !(model & operand);
                    cell.fetch_nand(operand, Ordering::SeqCst)
                }
                6 => {
                    model |= operand;
                    cell.fetch_or(operand, Ordering::Relaxed)
                }
                7 => {
                    model ^= operand;
                    cell.fetch_xor(operand, Ordering::AcqRel)
                }
                8 => {
                    if operand > model {
                        model = operand;
                    }
                    cell.fetch_max(operand, Ordering::SeqCst)
                }
                9 => {
                    if operand < model {
                        model = operand;
                    }
                    cell.fetch_min(operand, Ordering::Relaxed)
                }
                10 => {
                    model = operand;
                    cell.store(operand, Ordering::Release);
                    before
                }
                _ => {
                    // Against the current value, or a stale one for odd operands.
                    let expected = if operand as u64 & 1 == 0 { model } else { operand };
                    let replacement = operand.wrapping_add(1);
                    match cell.compare_exchange(expected, replacement, Ordering::SeqCst, Ordering::Acquire) {
                        Ok(seen) => {
                            bad += (expected != model) as u32;
                            model = replacement;
                            seen
                        }
                        Err(seen) => {
                            bad += (expected == model) as u32;
                            seen
                        }
                    }
                }
            };
            bad += (returned != before) as u32 + (cell.load(Ordering::Relaxed) != model) as u32;
            sum = sum.wrapping_mul(1_000_003).wrapping_add(returned as $u as u64);
            k += 1;
        }
        $out.check($bad, bad as u128, 0);
        $out.show($sum, sum as u128);
    }};
}

fn atomic_sweeps(out: &mut Out) {
    atomic_sweep!(out, "atomic u8 bad", "atomic u8 sum", AtomicU8, u8, u8);
    atomic_sweep!(out, "atomic i8 bad", "atomic i8 sum", AtomicI8, i8, u8);
    atomic_sweep!(out, "atomic u16 bad", "atomic u16 sum", AtomicU16, u16, u16);
    atomic_sweep!(out, "atomic i16 bad", "atomic i16 sum", AtomicI16, i16, u16);
    atomic_sweep!(out, "atomic u32 bad", "atomic u32 sum", AtomicU32, u32, u32);
    atomic_sweep!(out, "atomic i32 bad", "atomic i32 sum", AtomicI32, i32, u32);
    atomic_sweep!(out, "atomic u64 bad", "atomic u64 sum", AtomicU64, u64, u64);
    atomic_sweep!(out, "atomic i64 bad", "atomic i64 sum", AtomicI64, i64, u64);
    atomic_sweep!(out, "atomic usize bad", "atomic usize sum", AtomicUsize, usize, usize);
    atomic_sweep!(out, "atomic isize bad", "atomic isize sum", AtomicIsize, isize, usize);
}
