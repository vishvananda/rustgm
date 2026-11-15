#![no_std]
#![no_main]

use core::hint::black_box;
use core::mem::MaybeUninit;
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
    transfers(&mut out);
    out.flush();
    comparisons(&mut out);
    out.flush();
    extents(&mut out);
    out.flush();
    volatile(&mut out);
    out.flush();
    validity(&mut out);
    out.flush();
    floats(&mut out);
    out.flush();
    memory_sweeps(&mut out);
    out.flush();
    hints(&mut out);
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

/// `ptr::copy_nonoverlapping`, `ptr::copy` in both overlapping directions and
/// `ptr::write_bytes`, with elements of every representation.
fn transfers(out: &mut Out) {
    let source = black_box([1u32, 2, 3, 4, 5, 6]);
    let mut target = [0u32; 6];
    let t = target.as_mut_ptr();
    unsafe { ptr::copy_nonoverlapping(source.as_ptr(), t, 6) };
    out.check("copy u32", (target[0] + target[5] * 16) as u128, 0x61);
    unsafe { ptr::copy(t, t.add(2), 4) };
    out.check("copy up u32", (target[2] * 1000 + target[3] * 100 + target[4] * 10 + target[5]) as u128, 1234);
    unsafe { ptr::copy(t.add(1), t, 5) };
    out.check("copy down u32", (target[0] * 1000 + target[1] * 100 + target[4] * 10 + target[5]) as u128, 2144);
    unsafe { ptr::copy(t, t, 6) };
    out.check("copy in place", (target[2] * 10 + target[3]) as u128, 23);
    unsafe { ptr::copy_nonoverlapping(source.as_ptr(), t, 0) };
    out.check("copy none", target[0] as u128, 2);
    unsafe { ptr::write_bytes(t.add(1), 0xab, 2) };
    out.check("fill u32", ((target[1] as u64) << 32 | target[2] as u64) as u128, 0xabab_abab_abab_abab);
    out.check("fill u32 edges", (target[0] * 10 + target[3]) as u128, 23);
    unsafe { ptr::write_bytes(t, 0xff, 0) };
    out.check("fill none", target[0] as u128, 2);

    let bytes = black_box(*b"intrinsic");
    let mut copy = [0u8; 9];
    let c = copy.as_mut_ptr();
    unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), c, 9) };
    out.check("copy u8", (copy[0] as u32 * 256 + copy[8] as u32) as u128, 0x6963);
    unsafe { ptr::copy(c, c.add(1), 8) };
    out.check("copy up u8", (copy[1] as u32 * 256 + copy[8] as u32) as u128, 0x6969);
    unsafe { ptr::copy(c.add(2), c, 7) };
    out.check("copy down u8", (copy[0] as u32 * 256 + copy[6] as u32) as u128, 0x6e69);
    unsafe { ptr::write_bytes(c, 0, 9) };
    out.check("fill u8", (copy[0] | copy[4] | copy[8]) as u128, 0);

    let halves = black_box([0x1234u16, 0x5678, 0x9abc]);
    let mut copy = [0u16; 3];
    unsafe { ptr::copy_nonoverlapping(halves.as_ptr(), copy.as_mut_ptr(), 3) };
    out.check("copy u16", ((copy[0] as u64) << 32 | (copy[1] as u64) << 16 | copy[2] as u64) as u128, 0x1234_5678_9abc);
    unsafe { ptr::write_bytes(copy.as_mut_ptr().add(2), 0x5a, 1) };
    out.check("fill u16", ((copy[1] as u32) << 16 | copy[2] as u32) as u128, 0x5678_5a5a);

    let longs = black_box([u64::MAX, 1 << 40, 3]);
    let mut copy = [0u64; 3];
    unsafe { ptr::copy(longs.as_ptr(), copy.as_mut_ptr(), 3) };
    out.check("copy u64", (copy[0] ^ copy[1] ^ copy[2]) as u128, (u64::MAX ^ (1 << 40) ^ 3) as u128);

    let wide = black_box([1u128 << 100, 7, u128::MAX]);
    let mut copy = [0u128; 3];
    let c = copy.as_mut_ptr();
    unsafe { ptr::copy_nonoverlapping(wide.as_ptr(), c, 3) };
    out.check("copy u128", copy[0] ^ copy[1] ^ copy[2], u128::MAX ^ (1 << 100) ^ 7);
    unsafe { ptr::copy(c, c.add(1), 2) };
    out.check("copy up u128", copy[1] + copy[2], (1 << 100) + 7);
    unsafe { ptr::write_bytes(c, 0x11, 1) };
    out.check("fill u128", copy[0], u128::MAX / 15);

    let triples = black_box([[1u8, 2, 3], [4, 5, 6], [7, 8, 9]]);
    let mut copy = [[0u8; 3]; 3];
    let c = copy.as_mut_ptr();
    unsafe { ptr::copy_nonoverlapping(triples.as_ptr(), c, 3) };
    out.check("copy triple", (copy[0][0] as u32 * 100 + copy[1][1] as u32 * 10 + copy[2][2] as u32) as u128, 159);
    unsafe { ptr::copy(c, c.add(1), 2) };
    out.check("copy up triple", (copy[1][0] as u32 * 100 + copy[2][0] as u32 * 10 + copy[2][2] as u32) as u128, 146);
    unsafe { ptr::write_bytes(c, 5, 2) };
    out.check("fill triple", (copy[0][0] as u32 * 100 + copy[1][2] as u32 * 10 + copy[2][0] as u32) as u128, 554);

    let records = black_box([Record { a: 1, b: 2, c: [3; 5] }, Record { a: 4, b: 5, c: [6; 5] }]);
    let mut slots = [Record { a: 0, b: 0, c: [0; 5] }, Record { a: 0, b: 0, c: [0; 5] }];
    let s = slots.as_mut_ptr();
    unsafe { ptr::copy_nonoverlapping(records.as_ptr(), s, 2) };
    out.check(
        "copy record",
        (slots[0].a * 1000 + slots[0].c[4] as u64 * 100 + slots[1].b as u64 * 10 + slots[1].c[0] as u64) as u128,
        1356,
    );
    unsafe { ptr::copy(s.add(1), s, 1) };
    out.check("copy down record", (slots[0].a * 100 + slots[0].b as u64 * 10 + slots[0].c[2] as u64) as u128, 456);
    unsafe { ptr::write_bytes(s, 0, 2) };
    out.check("fill record", (slots[0].a + slots[1].b as u64 + slots[1].c[4] as u64) as u128, 0);

    let texts: [&str; 2] = black_box(["ab", "cde"]);
    let mut slots: [&str; 2] = [""; 2];
    unsafe { ptr::copy_nonoverlapping(texts.as_ptr(), slots.as_mut_ptr(), 2) };
    out.check("copy str reference", (slots[0].len() * 10 + slots[1].len()) as u128, 23);
    let flags = black_box([true, false, true]);
    let mut slots = [false; 3];
    unsafe { ptr::copy(flags.as_ptr(), slots.as_mut_ptr(), 3) };
    out.check("copy bool", (slots[0] as u8 * 4 + slots[1] as u8 * 2 + slots[2] as u8) as u128, 5);
    let reals = black_box([1.5f64, -0.0]);
    let mut slots = [0f64; 2];
    unsafe { ptr::copy_nonoverlapping(reals.as_ptr(), slots.as_mut_ptr(), 2) };
    out.check("copy f64", (slots[0].to_bits() ^ slots[1].to_bits()) as u128, 0xbff8_0000_0000_0000);
    let units = black_box([(); 4]);
    let mut slots = [(); 4];
    let s = slots.as_mut_ptr();
    unsafe {
        ptr::copy_nonoverlapping(units.as_ptr(), s, 4);
        ptr::copy(s, s.add(1), 3);
        ptr::write_bytes(s, 1, 4);
    }
    out.check("unit transfers", slots.len() as u128, 4);

    let mut bytes = [7u8; 9];
    bytes.copy_from_slice(black_box(&[1, 2, 3, 4, 5, 6, 7, 8, 9]));
    out.check("copy_from_slice", (bytes[0] * 10 + bytes[8]) as u128, 19);
    bytes.copy_within(2..6, 4);
    out.check(
        "copy_within up",
        (bytes[4] as u32 * 1000 + bytes[5] as u32 * 100 + bytes[6] as u32 * 10 + bytes[7] as u32) as u128,
        3456,
    );
    bytes.copy_within(3.., 0);
    out.check(
        "copy_within down",
        (bytes[0] as u32 * 1000 + bytes[5] as u32 * 100 + bytes[6] as u32 * 10 + bytes[8] as u32) as u128,
        4959,
    );
    bytes.fill(3);
    out.check("fill slice", (bytes[0] + bytes[8]) as u128, 6);
    let mut words = [0u16; 4];
    words.clone_from_slice(black_box(&[10, 20, 30, 40]));
    out.check("clone_from_slice", (words[0] + words[3]) as u128, 50);
    let mut left = black_box([1u32, 2, 3]);
    let mut right = black_box([4u32, 5, 6]);
    left.swap_with_slice(&mut right);
    out.check("swap_with_slice", (left[0] * 100 + left[2] * 10 + right[1]) as u128, 462);
    let mut a = black_box(1u64);
    let mut b = black_box(2u64);
    mem::swap(&mut a, &mut b);
    out.check("swap u64", (a * 16 + b) as u128, 0x21);
    let mut values = black_box([5u8, 4, 3, 2, 1]);
    values.reverse();
    out.check("reverse", (values[0] * 16 + values[4]) as u128, 0x15);
    values.rotate_left(2);
    out.check("rotate_left", (values[0] * 16 + values[4]) as u128, 0x32);
}

/// Slice, array and string equality and byte-slice ordering: the first
/// unequal byte decides, read as unsigned, and a prefix orders first.
fn comparisons(out: &mut Out) {
    let left = black_box(*b"intrinsic");
    let right = black_box(*b"intrinsiC");
    out.check("bytes eq", (left[..] == right[..]) as u128, 0);
    out.check("bytes eq prefix", (left[..8] == right[..8]) as u128, 1);
    out.check("bytes eq empty", (left[..0] == right[9..]) as u128, 1);
    out.check("bytes eq lengths", (left[..8] == right[..7]) as u128, 0);
    out.check("bytes ne", (left[..] != right[..]) as u128, 1);
    out.check("array eq", (left == right) as u128, 0);
    out.check("array eq same", (left == black_box(*b"intrinsic")) as u128, 1);
    out.check("bytes lt", (left[..] < right[..]) as u128, 0);
    out.check("bytes gt", (left[..] > right[..]) as u128, 1);
    out.check("bytes prefix lt", (left[..4] < left[..5]) as u128, 1);
    out.check("bytes le same", (left[..] <= left[..]) as u128, 1);
    let high = black_box([0x80u8, 0]);
    let low = black_box([0x7fu8, 0xff]);
    out.check("bytes unsigned order", (high[..] > low[..]) as u128, 1);
    out.check("bytes cmp greater", high[..].cmp(&low[..]) as i8 as u8 as u128, 1);
    out.check("bytes cmp less", low[..].cmp(&high[..]) as i8 as u8 as u128, 0xff);
    out.check("bytes cmp equal", low[..].cmp(&low[..]) as i8 as u8 as u128, 0);
    out.check("bytes cmp prefix", low[..1].cmp(&low[..]) as i8 as u8 as u128, 0xff);
    out.check("str eq", (black_box("alpha") == "alpha") as u128, 1);
    out.check("str eq case", (black_box("alpha") == "alphA") as u128, 0);
    out.check("str lt", (black_box("alpha") < "beta") as u128, 1);
    out.check("str cmp prefix", black_box("alp").cmp("alpha") as i8 as u8 as u128, 0xff);
    out.check("str cmp greater", black_box("b").cmp("alpha") as i8 as u8 as u128, 1);
    let words = black_box([1u32, 2, 3, 4]);
    out.check("u32 array eq", (words == [1, 2, 3, 4]) as u128, 1);
    out.check("u32 array eq last", (words == [1, 2, 3, 5]) as u128, 0);
    out.check("u32 slice eq", (words[1..] == [2, 3, 4]) as u128, 1);
    out.check("u32 slice eq first", (words[1..] == [3, 3, 4]) as u128, 0);
    let wide = black_box([u128::MAX, 1 << 100]);
    out.check("u128 array eq", (wide == [u128::MAX, 1 << 100]) as u128, 1);
    out.check("u128 array eq top", (wide == [u128::MAX, 1 << 101]) as u128, 0);
    let units = black_box([(); 3]);
    out.check("unit array eq", (units == [(); 3]) as u128, 1);
    let flags = black_box([true, false]);
    out.check("bool array eq", (flags == [true, false]) as u128, 1);
    out.check("bool slice order", (flags[..] > [false, true][..]) as u128, 1);
    let signed = black_box([-1i8, 5]);
    out.check("i8 slice order", (signed[..] < [0i8, 0][..]) as u128, 1);
    out.check("i8 array eq", (signed == [-1, 5]) as u128, 1);
    let letters = black_box(['a', '\u{3bb}']);
    out.check("char array eq", (letters == ['a', '\u{3bb}']) as u128, 1);
    out.check("char array eq last", (letters == ['a', '\u{3bc}']) as u128, 0);
}

/// `size_of_val` and `align_of_val` of sized values, slices and strings.
fn extents(out: &mut Out) {
    let words: &[u16] = black_box(&[1, 2, 3]);
    out.check("size slice u16", mem::size_of_val(words) as u128, 6);
    out.check("align slice u16", mem::align_of_val(words) as u128, 2);
    out.check("size empty slice", mem::size_of_val(&words[..0]) as u128, 0);
    let wide: &[u128] = black_box(&[1, 2]);
    out.check("size slice u128", mem::size_of_val(wide) as u128, 32);
    out.check("align slice u128", mem::align_of_val(wide) as u128, 16);
    let triples: &[[u8; 3]] = black_box(&[[0; 3]; 5]);
    out.check("size slice triple", mem::size_of_val(triples) as u128, 15);
    out.check("align slice triple", mem::align_of_val(triples) as u128, 1);
    let units: &[()] = black_box(&[(); 7]);
    out.check("size slice unit", mem::size_of_val(units) as u128, 0);
    out.check("size str", mem::size_of_val(black_box("hello")) as u128, 5);
    out.check("align str", mem::align_of_val(black_box("hello")) as u128, 1);
    out.check("size u64", mem::size_of_val(&black_box(7u64)) as u128, 8);
    out.check("align u64", mem::align_of_val(&black_box(7u64)) as u128, 8);
    let record = Record { a: 1, b: 2, c: [3; 5] };
    out.check("size record", mem::size_of_val(black_box(&record)) as u128, 16);
    out.check("align record", mem::align_of_val(black_box(&record)) as u128, 8);
    out.check("size array", mem::size_of_val(black_box(&[0u32; 5])) as u128, 20);
    out.check("size slice reference", mem::size_of_val(&words) as u128, 16);
    out.check("size unit", mem::size_of_val(&()) as u128, 0);
    out.check("align unit", mem::align_of_val(&()) as u128, 1);
}

/// `read_volatile` and `write_volatile` of scalars of every width.
fn volatile(out: &mut Out) {
    let mut cell = black_box(0xa5u8);
    out.check("volatile read u8", unsafe { ptr::read_volatile(&cell) } as u128, 0xa5);
    unsafe { ptr::write_volatile(&mut cell, 0x5a) };
    out.check("volatile write u8", cell as u128, 0x5a);
    let mut cell = black_box(-2i16);
    out.check("volatile read i16", unsafe { ptr::read_volatile(&cell) } as u16 as u128, 0xfffe);
    unsafe { ptr::write_volatile(&mut cell, 300) };
    out.check("volatile write i16", cell as u128, 300);
    let mut cell = black_box(5u32);
    unsafe { ptr::write_volatile(&mut cell, 9) };
    out.check("volatile u32", unsafe { ptr::read_volatile(&cell) } as u128, 9);
    let mut cell = black_box(1u64 << 40);
    out.check("volatile read u64", unsafe { ptr::read_volatile(&cell) } as u128, 1 << 40);
    unsafe { ptr::write_volatile(&mut cell, u64::MAX) };
    out.check("volatile write u64", cell as u128, u64::MAX as u128);
    let mut cell = black_box(1u128 << 100);
    out.check("volatile read u128", unsafe { ptr::read_volatile(&cell) }, 1 << 100);
    unsafe { ptr::write_volatile(&mut cell, u128::MAX - 1) };
    out.check("volatile write u128", cell, u128::MAX - 1);
    let mut cell = black_box(false);
    unsafe { ptr::write_volatile(&mut cell, true) };
    out.check("volatile bool", unsafe { ptr::read_volatile(&cell) } as u128, 1);
    let mut cell = black_box(1.5f64);
    out.check("volatile read f64", unsafe { ptr::read_volatile(&cell) }.to_bits() as u128, 0x3ff8_0000_0000_0000);
    unsafe { ptr::write_volatile(&mut cell, -2.5) };
    out.check("volatile write f64", cell.to_bits() as u128, 0xc004_0000_0000_0000);
    let target = 77u32;
    let mut cell: *const u32 = ptr::null();
    unsafe { ptr::write_volatile(&mut cell, black_box(&target)) };
    out.check("volatile pointer", unsafe { *ptr::read_volatile(&cell) } as u128, 77);
}

/// `mem::zeroed` and `MaybeUninit::assume_init` for types whose zero-valid
/// and inhabited assertions hold.
fn validity(out: &mut Out) {
    let zero: u64 = unsafe { mem::zeroed() };
    out.check("zeroed u64", zero as u128, 0);
    let zero: (u8, [u16; 3], bool, char) = unsafe { mem::zeroed() };
    out.check("zeroed tuple", (zero.0 as u32 + zero.1[2] as u32 + zero.2 as u32 + zero.3 as u32) as u128, 0);
    let zero: *const u32 = unsafe { mem::zeroed() };
    out.check("zeroed pointer", zero.is_null() as u128, 1);
    let zero: f64 = unsafe { mem::zeroed() };
    out.check("zeroed f64", zero.to_bits() as u128, 0);
    let zero: [u128; 2] = unsafe { mem::zeroed() };
    out.check("zeroed u128 array", zero[0] | zero[1], 0);
    let mut slot = MaybeUninit::<u32>::uninit();
    slot.write(black_box(77));
    out.check("assume_init u32", unsafe { slot.assume_init() } as u128, 77);
    let mut slot = MaybeUninit::<Record>::uninit();
    slot.write(Record { a: 1, b: 2, c: [3; 5] });
    let record = unsafe { slot.assume_init() };
    out.check("assume_init record", (record.a * 100 + record.b as u64 * 10 + record.c[1] as u64) as u128, 123);
    let slot = MaybeUninit::<[u8; 4]>::zeroed();
    out.check("zeroed assume_init", unsafe { slot.assume_init() }[3] as u128, 0);
    let slot = MaybeUninit::<Option<NonZero<u32>>>::zeroed();
    out.check("zeroed option", unsafe { slot.assume_init() }.is_none() as u128, 1);
}

/// `abs` clears the sign bit and `copysign` replaces it; every other bit,
/// a NaN payload included, is kept.
fn floats(out: &mut Out) {
    out.check("f64 abs", black_box(-2.5f64).abs().to_bits() as u128, 0x4004_0000_0000_0000);
    out.check("f64 abs positive", black_box(2.5f64).abs().to_bits() as u128, 0x4004_0000_0000_0000);
    out.check("f64 abs -0", black_box(-0.0f64).abs().to_bits() as u128, 0);
    out.check("f64 abs -inf", black_box(f64::NEG_INFINITY).abs().to_bits() as u128, 0x7ff0_0000_0000_0000);
    let nan = f64::from_bits(black_box(0xfff8_0000_0000_1234));
    out.check("f64 abs nan", nan.abs().to_bits() as u128, 0x7ff8_0000_0000_1234);
    out.check("f32 abs", black_box(-3.75f32).abs().to_bits() as u128, 0x4070_0000);
    out.check("f32 abs -0", black_box(-0.0f32).abs().to_bits() as u128, 0);
    let nan = f32::from_bits(black_box(0xffc0_0123));
    out.check("f32 abs nan", nan.abs().to_bits() as u128, 0x7fc0_0123);
    out.check("f64 copysign", black_box(-2.5f64).copysign(1.0).to_bits() as u128, 0x4004_0000_0000_0000);
    out.check("f64 copysign -0", black_box(2.5f64).copysign(-0.0).to_bits() as u128, 0xc004_0000_0000_0000);
    let nan = f64::from_bits(black_box(0xfff8_0000_0000_0000));
    out.check("f64 copysign nan", black_box(1.0f64).copysign(nan).to_bits() as u128, 0xbff0_0000_0000_0000);
    out.check("f32 copysign", black_box(3.75f32).copysign(-1.0).to_bits() as u128, 0xc070_0000);
    out.check("f32 copysign 0", black_box(-3.75f32).copysign(0.0).to_bits() as u128, 0x4070_0000);
    out.check("f64 signum", black_box(-2.5f64).signum().to_bits() as u128, 0xbff0_0000_0000_0000);
    out.check("f32 signum", black_box(0.0f32).signum().to_bits() as u128, 0x3f80_0000);
}

/// Overlapping copies, fills and byte comparisons at sampled offsets and
/// lengths beside loops over a model of the same bytes. `bad` counts
/// differences; `sum` is a checksum of the buffer after every step.
fn memory_sweeps(out: &mut Out) {
    let mut state = 0x0f1e_2d3c_4b5a_6978u64;
    let mut buffer = [0u8; 48];
    let mut model = [0u8; 48];
    let mut i = 0;
    while i < 48 {
        buffer[i] = (next(&mut state) & 3) as u8;
        model[i] = buffer[i];
        i += 1;
    }
    let mut bad = 0u32;
    let mut sum = 0u64;
    let mut k = 0;
    while k < 600 {
        let r = next(&mut state);
        let from = (r % 48) as usize;
        let to = ((r >> 8) % 48) as usize;
        let room = 48 - if from > to { from } else { to };
        let count = ((r >> 16) as usize) % (room + 1);
        let p = buffer.as_mut_ptr();
        match (r >> 40) % 3 {
            0 => {
                unsafe { ptr::copy(p.add(from), p.add(to), count) };
                let mut held = [0u8; 48];
                let mut i = 0;
                while i < count {
                    held[i] = model[from + i];
                    i += 1;
                }
                let mut i = 0;
                while i < count {
                    model[to + i] = held[i];
                    i += 1;
                }
            }
            1 => {
                let value = ((r >> 48) & 3) as u8;
                unsafe { ptr::write_bytes(p.add(to), value, count) };
                let mut i = 0;
                while i < count {
                    model[to + i] = value;
                    i += 1;
                }
            }
            _ => {
                let order = buffer[from..from + count].cmp(&buffer[to..to + count]) as i8;
                let equal = buffer[from..from + count] == buffer[to..to + count];
                let mut expected = 0i8;
                let mut i = 0;
                while i < count && expected == 0 {
                    if model[from + i] != model[to + i] {
                        expected = if model[from + i] < model[to + i] { -1 } else { 1 };
                    }
                    i += 1;
                }
                bad += (order != expected) as u32 + (equal != (expected == 0)) as u32;
                sum = sum.wrapping_add(order as u8 as u64);
            }
        }
        let mut i = 0;
        while i < 48 {
            bad += (buffer[i] != model[i]) as u32;
            sum = sum.wrapping_mul(31).wrapping_add(buffer[i] as u64);
            i += 1;
        }
        k += 1;
    }
    out.check("memory sweep bad", bad as u128, 0);
    out.show("memory sweep sum", sum as u128);
}

static DROPS: AtomicUsize = AtomicUsize::new(0);

struct Counted(u32);

impl Drop for Counted {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

/// `spin_loop` has no effect a program can observe. `select_unpredictable`
/// returns one value and drops the other once; `mem::forget` drops nothing.
fn hints(out: &mut Out) {
    core::hint::spin_loop();
    out.check("select true", core::hint::select_unpredictable(black_box(true), 5u32, 7u32) as u128, 5);
    out.check("select false", core::hint::select_unpredictable(black_box(false), 5u32, 7u32) as u128, 7);
    let picked = core::hint::select_unpredictable(
        black_box(false),
        Record { a: 1, b: 2, c: [3; 5] },
        Record { a: 4, b: 5, c: [6; 5] },
    );
    out.check("select record", (picked.a * 100 + picked.b as u64 * 10 + picked.c[0] as u64) as u128, 456);
    let picked = core::hint::select_unpredictable(black_box(true), Counted(1), Counted(2));
    out.check("select drops the other", (picked.0 as usize * 16 + DROPS.load(Ordering::Relaxed)) as u128, 0x11);
    drop(picked);
    out.check("select result dropped", DROPS.load(Ordering::Relaxed) as u128, 2);
    mem::forget(Counted(3));
    out.check("forget drops nothing", DROPS.load(Ordering::Relaxed) as u128, 2);
}
