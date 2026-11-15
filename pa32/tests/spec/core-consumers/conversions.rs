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

use core::num::{IntErrorKind, NonZero, Saturating, Wrapping};
use core::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
struct Celsius(i32);

#[derive(Debug, PartialEq, Clone, Copy)]
struct Kelvin(u32);

impl From<Kelvin> for Celsius {
    fn from(value: Kelvin) -> Self {
        Celsius(value.0 as i32 - 273)
    }
}

impl TryFrom<Celsius> for Kelvin {
    type Error = &'static str;

    fn try_from(value: Celsius) -> Result<Self, Self::Error> {
        u32::try_from(value.0 + 273).map(Kelvin).map_err(|_| "below absolute zero")
    }
}

impl FromStr for Celsius {
    type Err = core::num::ParseIntError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        text.trim_end_matches('C').parse().map(Celsius)
    }
}

fn widen<T: Into<u64>>(value: T) -> u64 {
    value.into()
}

fn bytes_of<T: AsRef<[u8]>>(value: T) -> usize {
    value.as_ref().iter().map(|b| *b as usize).sum()
}

fn decode(bytes: &[u8]) -> Result<&str, core::str::Utf8Error> {
    core::str::from_utf8(bytes)
}

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "into {} {} {} {}", widen(7u8), widen(60000u16), widen(true), widen('A' as u32))?;
    writeln!(out, "from {} {} {} {}", i64::from(-5i8), u128::from(u64::MAX), i128::from(i64::MIN), usize::from(9u16))?;
    writeln!(out, "from {} {} {:?}", u32::from('€'), u8::from(false), char::from(65u8))?;
    writeln!(
        out,
        "try_from {:?} {:?} {:?} {:?}",
        u8::try_from(255i32),
        u8::try_from(256i32).is_err(),
        i8::try_from(-129i64).is_err(),
        u64::try_from(-1i128).is_err()
    )?;
    writeln!(
        out,
        "try_from {:?} {:?} {:?}",
        usize::try_from(u64::MAX),
        i32::try_from(u32::MAX).map_err(|e| e == u8::try_from(256u16).unwrap_err()),
        char::try_from(0xD800u32).is_err()
    )?;
    writeln!(out, "try_into {:?}", <[u8; 4]>::try_from(&[1u8, 2, 3, 4][..]).map(u32::from_le_bytes))?;
    writeln!(out, "try_into {:?}", <&[u8; 2]>::try_from(&[1u8, 2, 3][..]).is_err())?;
    writeln!(out, "as {} {} {} {}", 300i32 as u8, -1i32 as u32, -1i8 as i64, 0xffff_ffffu32 as i32)?;
    writeln!(out, "as {} {} {} {}", u64::MAX as u16, -129i32 as i8, 200u8 as i8, i128::MIN as u8)?;
    writeln!(out, "as {} {} {:?} {}", 'a' as u8, 97u8 as char, 0x1F600u32 as u8 as char, true as i32)?;
    writeln!(out, "char {:?} {:?} {:?}", char::from_u32(0x41), char::from_u32(0x110000), char::from_u32(0xDFFF))?;
    writeln!(out, "digit {:?} {:?} {:?} {:?}", char::from_digit(7, 10), char::from_digit(11, 16), char::from_digit(10, 10), 'f'.to_digit(16))?;
    writeln!(out, "bytes {:?} {:?} {:?}", 0x1234_5678u32.to_be_bytes(), 0x1234u16.to_le_bytes(), (-2i32).to_ne_bytes().len())?;
    writeln!(out, "bytes {} {} {}", u32::from_be_bytes([1, 2, 3, 4]), i16::from_le_bytes([0xff, 0x7f]), u64::from_le_bytes([1, 0, 0, 0, 0, 0, 0, 1]))?;
    for text in ["42", "-17", "+8", "0", "", "-", "12a", "256", "-129", " 1", "00012", "340282366920938463463374607431768211456"] {
        writeln!(
            out,
            "parse {:?} u8={:?} i8={:?} u128={:?}",
            text,
            text.parse::<u8>().map_err(|e| e.kind().clone()),
            text.parse::<i8>().ok(),
            text.parse::<u128>().is_ok()
        )?;
    }
    let overflow = "99999999999".parse::<i32>().unwrap_err();
    writeln!(out, "error {} | {:?} | {}", overflow, overflow, *overflow.kind() == IntErrorKind::PosOverflow)?;
    writeln!(out, "error {} | {}", "".parse::<u8>().unwrap_err(), "x".parse::<u8>().unwrap_err())?;
    writeln!(out, "radix {:?} {:?} {:?} {:?}", u32::from_str_radix("ff", 16), i64::from_str_radix("-1010", 2), u8::from_str_radix("zz", 36).is_err(), u16::from_str_radix("777", 8))?;
    writeln!(out, "bool {:?} {:?} {}", "true".parse::<bool>(), "false".parse::<bool>(), "yes".parse::<bool>().unwrap_err())?;
    writeln!(out, "char {:?} {} {}", "é".parse::<char>(), "ab".parse::<char>().unwrap_err(), "".parse::<char>().unwrap_err())?;
    writeln!(out, "custom {:?} {:?} {:?}", "21C".parse::<Celsius>(), Celsius::from_str("-300"), "warm".parse::<Celsius>().is_err())?;
    writeln!(out, "custom {:?} {:?} {:?}", Celsius::from(Kelvin(300)), Kelvin::try_from(Celsius(-273)), Kelvin::try_from(Celsius(-274)))?;
    let kelvin: Result<Kelvin, _> = Celsius(27).try_into();
    let celsius: Celsius = Kelvin(0).into();
    writeln!(out, "custom {:?} {:?}", kelvin, celsius)?;
    writeln!(out, "as_ref {} {} {}", bytes_of("abc"), bytes_of([1u8, 2, 3]), bytes_of(&[250u8, 250][..]))?;
    writeln!(out, "utf8 {:?}", core::str::from_utf8(&[0xe2, 0x82, 0xac, b'!']))?;
    let invalid = decode(&[b'o', b'k', 0xff, b'x']).unwrap_err();
    writeln!(out, "utf8 error {} | {} {:?}", invalid, invalid.valid_up_to(), invalid.error_len())?;
    let truncated = decode(&[b'a', 0xe2, 0x82]).unwrap_err();
    writeln!(out, "utf8 error {} | {} {:?}", truncated, truncated.valid_up_to(), truncated.error_len())?;
    let mut buffer = [0u8; 4];
    writeln!(out, "encode {:?} {:?} {}", '€'.encode_utf8(&mut buffer).as_bytes(), '𝄞'.len_utf8(), '𝄞'.len_utf16())?;
    let mut units = [0u16; 2];
    writeln!(out, "utf16 {:?} {:?}", '𝄞'.encode_utf16(&mut units), char::decode_utf16([0xD834, 0xDD1E, 0x41]).map(|r| r.ok()).nth(0))?;
    let non_zero = NonZero::new(12u32);
    writeln!(out, "nonzero {:?} {:?} {:?}", non_zero.map(NonZero::get), NonZero::new(0i8), non_zero.and_then(|n| n.checked_mul(n)).map(NonZero::<u64>::from))?;
    writeln!(out, "nonzero {} {}", size_of::<Option<NonZero<u64>>>(), NonZero::<u8>::MAX.leading_zeros())?;
    let mut wrapping = Wrapping(250u8);
    wrapping += Wrapping(10);
    wrapping = wrapping * Wrapping(3) - Wrapping(20);
    writeln!(out, "wrapping {} {:?} {}", wrapping, Wrapping(i32::MAX) + Wrapping(1), (-Wrapping(i8::MIN)).0)?;
    writeln!(out, "saturating {} {} {}", Saturating(250u8) + Saturating(10), Saturating(-100i8) - Saturating(100), Saturating(i32::MAX) * Saturating(2))?;
    writeln!(out, "option {:?} {:?} {:?}", Option::from(3), Some(4u8).map(u32::from), <Option<&str>>::from(None::<&str>))?;
    let tuple: (u8, char) = (1, 'x');
    let array: [u16; 3] = [1u8, 2, 3].map(u16::from);
    let from_tuple: [i32; 2] = (5, 6).into();
    writeln!(out, "aggregate {:?} {:?} {:?}", tuple, array, from_tuple)?;
    writeln!(out, "str {} {:?} {}", <&str>::from("x"), "bytes".as_bytes(), unsafe { core::str::from_utf8_unchecked(b"raw") })?;
    Ok(())
}
