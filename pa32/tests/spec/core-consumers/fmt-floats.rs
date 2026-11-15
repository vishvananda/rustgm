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

fn bits(value: f64) -> u64 {
    value.to_bits()
}

fn run(out: &mut Out) -> fmt::Result {
    let values = [
        0.0f64,
        -0.0,
        1.0,
        -1.5,
        0.1,
        0.2 + 0.1,
        1.0 / 3.0,
        123456789.125,
        1e15,
        1e16,
        1e21,
        1.5e300,
        1e-5,
        1e-7,
        5e-324,
        f64::MAX,
        f64::MIN_POSITIVE,
        f64::EPSILON,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NAN,
    ];
    for value in values {
        writeln!(out, "{} | {:?} | {:e} | {:.3} | {:x}", value, value, value, value, bits(value))?;
    }
    let singles = [0.1f32, 16777216.0, 16777217.0, 3.4028235e38, 1e-45, 1.17549435e-38, 0.3, 2.5];
    for value in singles {
        writeln!(out, "{} | {:?} | {:E} | {:.2} | {:08x}", value, value, value, value, value.to_bits())?;
    }
    writeln!(out, "[{:10.3}] [{:<10.3}] [{:^10.3}] [{:+.1}] [{:010.2}]", 3.14159, 2.71828, 1.41421, 1.0, -2.5)?;
    writeln!(out, "[{:.0}] [{:.0}] [{:.0}] [{:.0}] [{:.1}]", 0.5, 1.5, 2.5, -0.5, 0.25)?;
    writeln!(out, "[{:8.2e}] [{:.0e}] [{:+e}] [{:12.4E}]", 123456.0, 15000.0, 0.00012, 6.02214076e23)?;
    writeln!(out, "[{:5}] [{:<5}] [{:>8}] [{:08.3}]", 1.0, 2.5, f64::NAN, f64::INFINITY)?;
    writeln!(out, "{} {} {}", 1e7, 1.0e-4, 123456789012345680.0)?;
    writeln!(out, "{:?} {:?} {:?}", 1e7, 1.0e-4, 123456789012345680.0)?;

    let a = 7.5f64;
    let b = -2.25f64;
    writeln!(out, "arith {} {} {} {} {}", a + b, a - b, a * b, a / b, a % b)?;
    writeln!(out, "neg abs {} {} {} {}", -a, b.abs(), b.signum(), a.copysign(b))?;
    writeln!(out, "minmax {} {} {} {}", a.min(b), a.max(b), a.clamp(0.0, 5.0), f64::NAN.max(1.0))?;
    let nan = f64::NAN;
    writeln!(out, "cmp {} {} {} {:?} {:?}", a > b, a == a, nan == nan, a.partial_cmp(&b), nan.partial_cmp(&a))?;
    writeln!(out, "total_cmp {:?} {:?}", (-0.0f64).total_cmp(&0.0), f64::NAN.total_cmp(&f64::INFINITY))?;
    writeln!(
        out,
        "classify {:?} {:?} {:?} {:?} {:?}",
        a.classify(),
        0.0f64.classify(),
        5e-324f64.classify(),
        f64::NAN.classify(),
        f64::INFINITY.classify()
    )?;
    writeln!(
        out,
        "predicates {} {} {} {} {}",
        f64::NAN.is_nan(),
        f64::INFINITY.is_finite(),
        (-0.0f64).is_sign_negative(),
        1e-320f64.is_subnormal(),
        a.is_normal()
    )?;
    writeln!(out, "recip {} degrees {} radians {}", 4.0f64.recip(), core::f64::consts::PI.to_degrees(), 90.0f32.to_radians())?;
    writeln!(out, "consts {} {} {}", core::f64::consts::E, core::f32::consts::SQRT_2, core::f64::consts::LN_2)?;
    writeln!(out, "bits {:#x} {:#x} {}", 1.0f32.to_bits(), (-2.0f64).to_bits(), f64::from_bits(0x4009_21fb_5444_2d18))?;
    writeln!(out, "bytes {:?} {}", 1.5f32.to_le_bytes(), f64::from_be_bytes([0x40, 0x45, 0, 0, 0, 0, 0, 0]))?;

    writeln!(out, "to int {} {} {} {} {}", 3.99f64 as i32, -3.99f64 as i32, 300.0f32 as u8, -1.0f32 as u8, f64::NAN as i64)?;
    writeln!(out, "saturate {} {} {}", 1e40f64 as i128, -1e40f64 as i128, 1e40f64 as u128)?;
    writeln!(out, "saturate {} {} {}", f64::INFINITY as u64, f64::NEG_INFINITY as i16, 1e10f32 as i32)?;
    writeln!(out, "from int {} {} {} {}", u64::MAX as f64, i64::MIN as f32, u128::MAX as f32, i128::MIN as f64)?;
    writeln!(out, "from int {} {} {}", 16777217i32 as f32, 9007199254740993u64 as f64, u128::MAX as f64)?;
    writeln!(out, "widen {} narrow {} {}", f64::from(0.1f32), 0.1f64 as f32, 1e300f64 as f32)?;
    writeln!(out, "lossless {} {} {}", f64::from(7u32), f32::from(-3i16), f64::from(true as u8))?;

    for text in ["3.25", "-0", "1e3", "1E-2", ".5", "5.", "inf", "-infinity", "NaN", "1e400", "1e-400", "0x10", "", "1.2.3", "179769313486231570000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000", "2.2250738585072011e-308", "0.1e1", "+7"] {
        match text.parse::<f64>() {
            Ok(value) => writeln!(out, "parse {:?} -> {:?} {:x}", &text[..text.len().min(12)], value, value.to_bits())?,
            Err(error) => writeln!(out, "parse {:?} -> error {}", text, error)?,
        }
    }
    for text in ["0.1", "16777217", "3.4028236e38", "1e-46"] {
        writeln!(out, "parse f32 {:?} -> {:?}", text, text.parse::<f32>())?;
    }
    let mut sum = 0.0f64;
    let mut term = 1.0f64;
    for n in 1..20 {
        sum += term;
        term /= n as f64;
    }
    writeln!(out, "series {} {:.15} {}", sum, sum, sum == core::f64::consts::E)?;
    let mut x = 2.0f64;
    for _ in 0..6 {
        x = 0.5 * (x + 2.0 / x);
    }
    writeln!(out, "newton {} {}", x, x * x)?;
    Ok(())
}
