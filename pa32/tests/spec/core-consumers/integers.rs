#![no_std]
#![no_main]

use core::fmt::{self, Write};

#[link(name = "c")]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn abort() -> !;
}

/// Buffered standard output through `core::fmt::Write` and libc `write`.
struct Out {
    buf: [u8; 256],
    len: usize,
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
}

impl Write for Out {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            if self.len == self.buf.len() {
                self.flush();
            }
            self.buf[self.len] = b;
            self.len += 1;
        }
        Ok(())
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    unsafe { abort() }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let mut out = Out { buf: [0; 256], len: 0 };
    let status = match run(&mut out) {
        Ok(()) => 0,
        Err(fmt::Error) => 1,
    };
    out.flush();
    status
}

fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

fn mod_pow(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut result = 1u64;
    base %= modulus;
    while exponent > 0 {
        if exponent & 1 == 1 {
            result = (u128::from(result) * u128::from(base) % u128::from(modulus)) as u64;
        }
        base = (u128::from(base) * u128::from(base) % u128::from(modulus)) as u64;
        exponent >>= 1;
    }
    result
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    for witness in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n % witness == 0 {
            return n == witness;
        }
    }
    let mut d = n - 1;
    let mut r = 0;
    while d % 2 == 0 {
        d /= 2;
        r += 1;
    }
    'witness: for witness in [2u64, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        let mut x = mod_pow(witness, d, n);
        if x == 1 || x == n - 1 {
            continue;
        }
        for _ in 0..r - 1 {
            x = (u128::from(x) * u128::from(x) % u128::from(n)) as u64;
            if x == n - 1 {
                continue 'witness;
            }
        }
        return false;
    }
    true
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D)
}

fn collatz(mut n: u64) -> u32 {
    let mut steps = 0;
    while n != 1 {
        n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };
        steps += 1;
    }
    steps
}

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "wrapping {} {} {} {}", 250u8.wrapping_add(10), 0u8.wrapping_sub(1), i32::MAX.wrapping_add(1), i8::MIN.wrapping_neg())?;
    writeln!(out, "wrapping {} {} {}", 200u8.wrapping_mul(3), i64::MIN.wrapping_abs(), 1u32.wrapping_shl(35))?;
    writeln!(out, "checked {:?} {:?} {:?} {:?}", 250u8.checked_add(5), 250u8.checked_add(6), i32::MIN.checked_abs(), 5u8.checked_sub(6))?;
    writeln!(out, "checked {:?} {:?} {:?} {:?}", 7i32.checked_div(0), i32::MIN.checked_div(-1), 7u32.checked_rem(2), 2u64.checked_pow(64))?;
    writeln!(out, "checked {:?} {:?} {:?}", 1u8.checked_shl(8), i16::MAX.checked_mul(2), i8::MIN.checked_neg())?;
    writeln!(out, "saturating {} {} {} {}", 250u8.saturating_add(10), 5u8.saturating_sub(6), i32::MIN.saturating_sub(1), i64::MAX.saturating_mul(2))?;
    writeln!(out, "saturating {} {}", i8::MIN.saturating_abs(), 3u16.saturating_pow(20))?;
    writeln!(out, "overflowing {:?} {:?} {:?}", 250u8.overflowing_add(10), i32::MAX.overflowing_mul(2), 5i8.overflowing_sub(-128))?;
    writeln!(out, "pow {} {} {} {}", 2u64.pow(40), (-3i32).pow(5), 10u128.pow(30), 7u8.pow(0))?;
    writeln!(out, "roots {} {} {} {}", 99u32.isqrt(), u64::MAX.isqrt(), 1000u32.ilog10(), 1u64.ilog2())?;
    writeln!(out, "logs {} {:?} {}", 80u32.ilog(3), 0u32.checked_ilog2(), u128::MAX.ilog2())?;
    writeln!(out, "bits {} {} {} {}", 0b1011_0000u8.count_ones(), 1u32.leading_zeros(), 0x80u64.trailing_zeros(), u16::MAX.count_zeros())?;
    writeln!(out, "bits {} {} {}", 0u8.leading_zeros(), (-1i32).leading_ones(), 0b0111u8.trailing_ones())?;
    writeln!(out, "rotate {:#x} {:#x} {:#x}", 0x1234_5678u32.rotate_left(8), 0x12u8.rotate_right(4), 0x8000_0000_0000_0001u64.rotate_left(1))?;
    writeln!(out, "bytes {:#x} {:#x} {:#x}", 0x1234u16.swap_bytes(), 0x1234_5678u32.reverse_bits(), 0x0102_0304u32.to_be())?;
    writeln!(out, "shifts {} {} {} {}", 1u32 << 31, -16i32 >> 2, 0xf0u8 >> 4, (-1i64) as u64 >> 60)?;
    writeln!(out, "logic {:#b} {:#b} {:#b} {:#b}", 0b1100u8 & 0b1010, 0b1100u8 | 0b1010, 0b1100u8 ^ 0b1010, !0b1100u8)?;
    writeln!(out, "division {} {} {} {}", 7 / 2, -7 / 2, 7 % -3, -7 % 3)?;
    writeln!(out, "euclid {} {} {} {}", (-7i32).div_euclid(2), (-7i32).rem_euclid(3), 7i32.div_euclid(-2), 7i32.rem_euclid(-3))?;
    writeln!(out, "rounding {} {} {} {}", 7u32.div_ceil(2), 13u32.next_multiple_of(5), 17u32.next_power_of_two(), 16u32.is_power_of_two())?;
    writeln!(out, "abs {} {} {} {}", (-5i32).abs(), i8::MIN.unsigned_abs(), 3u8.abs_diff(250), (-100i8).abs_diff(100))?;
    writeln!(out, "sign {} {} {} {}", (-9i32).signum(), 0i32.signum(), 5i32.is_positive(), i64::MIN.is_negative())?;
    writeln!(out, "midpoint {} {} {}", 0u8.midpoint(255), i32::MIN.midpoint(i32::MAX), u64::MAX.midpoint(u64::MAX - 2))?;
    writeln!(out, "minmax {} {} {} {}", 3.min(9), 3.max(9), i128::MAX.min(0), u8::MAX.max(u8::MIN))?;

    let big = 0x0123_4567_89ab_cdef_0011_2233_4455_6677u128;
    writeln!(out, "u128 {} {}", big, big.wrapping_mul(big))?;
    writeln!(out, "u128 {} {} {}", big / 1_000_000_007, big % 1_000_000_007, big / (big >> 61))?;
    writeln!(out, "u128 {} {} {}", big >> 67, big << 3 >> 100, big.count_ones())?;
    writeln!(out, "u128 {} {} {}", big.leading_zeros(), big.trailing_zeros(), (big as u64).swap_bytes())?;
    let negative = -170141183460469231731687303715884105i128;
    writeln!(out, "i128 {} {} {}", negative / 12345, negative % 12345, negative >> 70)?;
    writeln!(out, "i128 {:?} {:?}", negative.checked_mul(1000), negative.checked_mul(1001))?;
    writeln!(out, "i128 {} {}", i128::MAX / -1, (i128::MIN + 1).abs() == i128::MAX)?;
    writeln!(out, "widening {} {}", u64::MAX as u128 * u64::MAX as u128, (i64::MIN as i128) * (i64::MIN as i128))?;
    writeln!(out, "narrowing {} {} {}", (big >> 64) as u64, big as u8, (negative as i64) as i128 == negative)?;

    writeln!(out, "gcd {} {} {}", gcd(1071, 462), gcd(u64::MAX, 2u64.pow(32) - 1), gcd(17, 0))?;
    writeln!(out, "mod_pow {} {} {}", mod_pow(2, 10, 1000), mod_pow(3, 1_000_000_006, 1_000_000_007), mod_pow(7, u64::MAX, 18_446_744_073_709_551_557))?;
    write!(out, "primes")?;
    for candidate in [1u64, 2, 91, 97, 561, 7919, 1_000_000_007, 1_000_000_011, 18_446_744_073_709_551_557, 18_446_744_073_709_551_559] {
        write!(out, " {}", is_prime(candidate))?;
    }
    writeln!(out)?;
    let mut sieve = [true; 200];
    sieve[0] = false;
    sieve[1] = false;
    let mut i = 2;
    while i * i < sieve.len() {
        if sieve[i] {
            let mut multiple = i * i;
            while multiple < sieve.len() {
                sieve[multiple] = false;
                multiple += i;
            }
        }
        i += 1;
    }
    writeln!(out, "sieve {} last {:?}", sieve.iter().filter(|p| **p).count(), sieve.iter().rposition(|p| *p))?;
    let mut state = 88172645463325252u64;
    let mut mix = 0u64;
    let mut buckets = [0u32; 8];
    for _ in 0..1000 {
        let value = xorshift(&mut state);
        mix ^= value.rotate_left((value & 63) as u32);
        buckets[(value >> 61) as usize] += 1;
    }
    writeln!(out, "xorshift {:016x} {:016x} {:?}", state, mix, buckets)?;
    writeln!(out, "collatz {} {} {}", collatz(27), collatz(97), (1..30u64).map(collatz).max().unwrap_or(0))?;
    let mut digits = [0u8; 40];
    let mut value = u128::MAX;
    let mut count = 0;
    while value > 0 {
        digits[count] = (value % 10) as u8;
        value /= 10;
        count += 1;
    }
    writeln!(out, "digits {} sum {}", count, digits[..count].iter().map(|d| u32::from(*d)).sum::<u32>())?;
    let sizes = (i8::BITS, u16::BITS, i32::BITS, u64::BITS, i128::BITS, usize::BITS);
    writeln!(out, "bits {:?}", sizes)?;
    let mixed = (7u8 as i8 as i32 - 300) as u16 as u64 * 3;
    let precedence = 2 + 3 * 4 - 10 / 3 % 2 << 1 & 0xff | 1 ^ 8;
    writeln!(out, "mixed {} {} {}", mixed, precedence, -2i32.pow(2))?;
    Ok(())
}
