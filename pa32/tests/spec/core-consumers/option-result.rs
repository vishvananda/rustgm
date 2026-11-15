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

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParseError {
    Empty,
    BadDigit(u8),
    Overflow,
}

fn parse_u8(text: &str) -> Result<u8, ParseError> {
    if text.is_empty() {
        return Err(ParseError::Empty);
    }
    let mut value: u8 = 0;
    for byte in text.bytes() {
        let digit = match byte {
            b'0'..=b'9' => byte - b'0',
            other => return Err(ParseError::BadDigit(other)),
        };
        value = value
            .checked_mul(10)
            .and_then(|v| v.checked_add(digit))
            .ok_or(ParseError::Overflow)?;
    }
    Ok(value)
}

fn sum_pair(a: &str, b: &str) -> Result<u16, ParseError> {
    let x = parse_u8(a)?;
    let y = parse_u8(b)?;
    Ok(u16::from(x) + u16::from(y))
}

fn first_even(values: &[i32]) -> Option<&i32> {
    values.iter().find(|v| **v % 2 == 0)
}

fn run(out: &mut Out) -> fmt::Result {
    let some: Option<i32> = Some(20);
    let none: Option<i32> = None;
    writeln!(out, "map {:?} {:?}", some.map(|v| v + 1), none.map(|v| v + 1))?;
    writeln!(
        out,
        "and_then {:?} {:?}",
        some.and_then(|v| v.checked_mul(2)),
        some.and_then(|v| v.checked_mul(i32::MAX))
    )?;
    writeln!(out, "or {:?} {:?} {:?}", none.or(Some(7)), none.or_else(|| Some(8)), some.xor(none))?;
    writeln!(
        out,
        "unwrap_or {} {} {}",
        none.unwrap_or(5),
        none.unwrap_or_else(|| 6),
        none.unwrap_or_default()
    )?;
    writeln!(out, "filter {:?} {:?}", some.filter(|v| *v > 10), some.filter(|v| *v > 30))?;
    writeln!(out, "zip {:?} {:?}", some.zip(Some('z')), some.zip(none))?;
    writeln!(out, "ok_or {:?} {:?}", some.ok_or("none"), none.ok_or("none"))?;
    writeln!(out, "map_or {} {}", some.map_or(0, |v| v * 3), none.map_or_else(|| -1, |v| v * 3))?;
    writeln!(
        out,
        "is {} {} {}",
        some.is_some_and(|v| v == 20),
        none.is_none(),
        some.is_none_or(|v| v < 0)
    )?;
    let mut slot = Some(3);
    let taken = slot.take();
    writeln!(out, "take {:?} {:?}", taken, slot)?;
    let old = slot.replace(9);
    writeln!(out, "replace {:?} {:?}", old, slot)?;
    *slot.get_or_insert(1) += 1;
    writeln!(out, "get_or_insert {:?}", slot)?;
    let mut empty: Option<u8> = None;
    *empty.get_or_insert_with(|| 40) += 2;
    writeln!(out, "get_or_insert_with {:?}", empty)?;
    if let Some(inner) = slot.as_mut() {
        *inner *= 10;
    }
    writeln!(out, "as_mut {:?} as_ref {:?}", slot, slot.as_ref().map(|v| *v + 1))?;
    let nested: Option<Option<u8>> = Some(Some(4));
    writeln!(out, "flatten {:?} copied {:?}", nested.flatten(), first_even(&[1, 3, 6, 8]).copied())?;
    let transposed: Result<Option<u8>, ParseError> = Some(parse_u8("12")).transpose();
    writeln!(out, "transpose {:?}", transposed)?;
    writeln!(out, "iter {} {}", some.iter().count(), none.into_iter().chain(some).sum::<i32>())?;

    for text in ["42", "", "25x", "256", "255"] {
        writeln!(out, "parse {:?} -> {:?}", text, parse_u8(text))?;
    }
    writeln!(
        out,
        "sum {:?} {:?} {:?}",
        sum_pair("200", "100"),
        sum_pair("1", "?"),
        sum_pair("", "1")
    )?;
    let good: Result<u8, ParseError> = Ok(3);
    let bad: Result<u8, ParseError> = Err(ParseError::Empty);
    writeln!(out, "result map {:?} {:?}", good.map(|v| v * 2), bad.map(|v| v * 2))?;
    writeln!(out, "map_err {:?}", bad.map_err(|e| e == ParseError::Empty))?;
    writeln!(
        out,
        "and_then {:?} {:?}",
        good.and_then(|v| parse_u8("9").map(|w| v + w)),
        good.and(bad)
    )?;
    writeln!(out, "or_else {:?} unwrap_or {}", bad.or_else(|_| parse_u8("77")), bad.unwrap_or(1))?;
    writeln!(out, "ok {:?} err {:?} {:?}", good.ok(), bad.err(), good.err())?;
    writeln!(
        out,
        "is {} {} {}",
        good.is_ok(),
        bad.is_err_and(|e| e == ParseError::Overflow),
        good.is_ok_and(|v| v == 3)
    )?;
    let total: Result<u32, ParseError> = ["1", "2", "3"].iter().map(|t| parse_u8(t).map(u32::from)).sum();
    let failed: Result<u32, ParseError> = ["1", "x", "3"].iter().map(|t| parse_u8(t).map(u32::from)).sum();
    writeln!(out, "sum results {:?} {:?}", total, failed)?;
    let maybe: Option<u32> = [Some(1u32), Some(2), None].into_iter().sum();
    let product: Option<u32> = [Some(2u32), Some(5)].into_iter().product();
    writeln!(out, "sum options {:?} {:?}", maybe, product)?;
    Ok(())
}
