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

const LIMIT: usize = 12;
const GREETING: &str = "const str";
const MASKS: [u8; 4] = [1 << 0, 1 << 2, 1 << 4, 1 << 6];
const fn fibonacci(n: u32) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;
    let mut i = 0;
    while i < n {
        let next = a + b;
        a = b;
        b = next;
        i += 1;
    }
    a
}
const FIB_90: u64 = fibonacci(90);

const fn factorial(n: u128) -> u128 {
    if n == 0 { 1 } else { n * factorial(n - 1) }
}
const FACT_30: u128 = factorial(30);

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut index = 0;
    while index < 256 {
        let mut value = index as u32;
        let mut bit = 0;
        while bit < 8 {
            value = if value & 1 != 0 { 0xEDB8_8320 ^ (value >> 1) } else { value >> 1 };
            bit += 1;
        }
        table[index] = value;
        index += 1;
    }
    table
}
static CRC_TABLE: [u32; 256] = crc_table();

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc = CRC_TABLE[((crc ^ u32::from(*byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

const fn squares<const N: usize>() -> [usize; N] {
    let mut result = [0; N];
    let mut i = 0;
    while i < N {
        result[i] = i * i;
        i += 1;
    }
    result
}
static SQUARES: [usize; LIMIT] = squares();

struct Config {
    name: &'static str,
    retries: u8,
    ratio: f32,
}
const DEFAULT: Config = Config { name: "default", retries: 3, ratio: 0.5 };
static CONFIGS: [Config; 2] = [DEFAULT, Config { name: "eager", retries: 9, ..DEFAULT }];
static NAMES: &[&str] = &["zero", "one", "two"];
static FIRST_NAME: &&str = &NAMES[0];

trait Sized2 {
    const BYTES: usize;
    const DOUBLE: usize = Self::BYTES * 2;
}
impl Sized2 for u16 {
    const BYTES: usize = 2;
}
impl Sized2 for [u8; 5] {
    const BYTES: usize = 5;
    const DOUBLE: usize = 11;
}
struct Buffer<const N: usize>;
impl<const N: usize> Buffer<N> {
    const CAPACITY: usize = N * 8;
}

const fn pick(flag: bool) -> &'static str {
    match flag {
        true => "yes",
        false => "no",
    }
}
const PICKED: &str = pick(LIMIT > 10);
const WRAPPED: u8 = 250u8.wrapping_add(10);
const CHECKED: Option<u8> = 250u8.checked_add(10);
const SHIFTED: i32 = -64 >> 3;
const FLOAT: f64 = 1.5 * 4.0 - 0.25;
const CHAR: char = (b'a' + 3) as char;
const TUPLE: (u8, bool, &str) = (7, LIMIT == 12, GREETING);
const BYTES: &[u8; 5] = b"hello";
const LENGTH: usize = GREETING.len() + BYTES.len();

static mut SCRATCH: [u32; 4] = [1, 2, 3, 4];

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "{} {:?} {:?} {}", LIMIT, GREETING, MASKS, FIB_90)?;
    writeln!(out, "{} {}", FACT_30, fibonacci(20))?;
    writeln!(out, "crc {:08x} {:08x} {:08x}", crc32(b""), crc32(b"123456789"), crc32(GREETING.as_bytes()))?;
    writeln!(out, "table {:08x} {:08x} {:08x}", CRC_TABLE[1], CRC_TABLE[128], CRC_TABLE[255])?;
    writeln!(out, "squares {:?} {}", SQUARES, squares::<3>().len())?;
    for config in &CONFIGS {
        writeln!(out, "config {} {} {}", config.name, config.retries, config.ratio)?;
    }
    writeln!(out, "names {:?} {} {}", NAMES, FIRST_NAME, NAMES.len())?;
    writeln!(out, "assoc {} {} {} {}", u16::BYTES, u16::DOUBLE, <[u8; 5]>::BYTES, <[u8; 5]>::DOUBLE)?;
    writeln!(out, "generic {} {}", Buffer::<4>::CAPACITY, Buffer::<{ LIMIT + 1 }>::CAPACITY)?;
    writeln!(out, "{} {} {:?} {} {} {:?}", PICKED, WRAPPED, CHECKED, SHIFTED, FLOAT, CHAR)?;
    writeln!(out, "{:?} {:?} {}", TUPLE, BYTES, LENGTH)?;
    let array = [0u8; LIMIT / 3];
    let inline = const { fibonacci(10) as usize + LIMIT };
    writeln!(out, "inline {} {} {}", array.len(), inline, [const { 1u8 << 3 }; 2][1])?;
    let promoted: &'static [u16] = &[10, 20, 30];
    let literal: &'static str = match promoted.len() {
        LIMIT => "limit",
        3 => "three",
        _ => "other",
    };
    writeln!(out, "promoted {:?} {}", promoted, literal)?;
    writeln!(out, "limits {} {} {} {}", u8::MAX, i16::MIN, u32::BITS, usize::BITS)?;

    let scratch = unsafe {
        let pointer = &raw mut SCRATCH;
        (*pointer)[1] += 40;
        (*pointer).swap(0, 3);
        *pointer
    };
    writeln!(out, "static mut {:?}", scratch)?;
    Ok(())
}
