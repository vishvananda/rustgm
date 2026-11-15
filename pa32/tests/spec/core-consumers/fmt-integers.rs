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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Shape {
    Unit,
    Tuple(u8, char),
    Named { origin: Point, label: &'static str },
}

#[derive(Debug)]
#[allow(dead_code)]
struct Wrapper<'a, T>(&'a [T]);

struct Hex(u32);

impl fmt::Display for Hex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x{:08X}", self.0)
        } else {
            write!(f, "{:x}", self.0)
        }
    }
}

struct Padded(&'static str);

impl fmt::Display for Padded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.0)
    }
}

struct Listing;

impl fmt::Debug for Listing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries([1, 2, 3].iter()).finish()?;
        f.write_str(" ")?;
        f.debug_map().entry(&"k", &1).entry(&"l", &2).finish()?;
        f.write_str(" ")?;
        f.debug_set().entry(&'a').entry(&'b').finish()?;
        f.write_str(" ")?;
        f.debug_tuple("T").field(&1).field(&"two").finish()?;
        f.write_str(" ")?;
        f.debug_struct("S").field("a", &1).finish_non_exhaustive()
    }
}

struct Options;

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "width={:?} precision={:?} fill={:?} plus={} zero={}",
            f.width(),
            f.precision(),
            f.fill(),
            f.sign_plus(),
            f.sign_aware_zero_pad()
        )
    }
}

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "{} {} {} {}", 0u8, 255u8, -128i8, 127i8)?;
    writeln!(out, "{} {} {} {}", u16::MAX, i16::MIN, u32::MAX, i32::MIN)?;
    writeln!(out, "{} {}", u64::MAX, i64::MIN)?;
    writeln!(out, "{} {}", u128::MAX, i128::MIN)?;
    writeln!(out, "{} {}", usize::MAX, isize::MIN)?;
    writeln!(out, "[{:5}] [{:<5}] [{:^5}] [{:>5}]", 42, 42, 42, 42)?;
    writeln!(out, "[{:05}] [{:+}] [{:+05}] [{:<05}]", 42, 42, -42, 7)?;
    writeln!(out, "[{:*^9}] [{:->6}] [{:#<4}]", 123, -5, 1)?;
    writeln!(out, "{:x} {:X} {:o} {:b}", 255, 255, 8, 5)?;
    writeln!(out, "{:#x} {:#X} {:#o} {:#b}", 255, 255, 8, 5)?;
    writeln!(out, "{:#010x} {:#06b} {:08b} {:02X}", 0xbeefu32, 2, 37u8, 10)?;
    writeln!(out, "{:x} {:b} {:o}", -1i8, -2i16, -8i32)?;
    writeln!(out, "{:x} {:#X}", u128::MAX, i128::MIN)?;
    writeln!(out, "{:e} {:E} {:e} {:e}", 1234567u32, 1500u64, 0u8, -120i32)?;
    writeln!(out, "{:.2e} {:10.1e}|", 123456u32, 98765i64)?;
    let width = 8;
    let precision = 3;
    writeln!(out, "[{:width$}] [{:>1$}] [{:<w2$}]", 7, 6, 5, w2 = 4)?;
    writeln!(out, "[{:.*}] [{:.precision$}] [{:width$.precision$}]", 2, "abcdef", "abcdef", "abcdef")?;
    writeln!(out, "{0} {1} {0} {name} {name:?}", 1, 2, name = "n")?;
    writeln!(out, "{width} {precision:03} {{braces}} {}%", 100)?;
    writeln!(out, "[{:6}] [{:<6}] [{:^6}] [{:>6}]", true, 'c', "ab", false)?;
    writeln!(out, "[{:?}] [{:?}] [{:?}] [{:?}]", 'q', "q\"uote\n", '\'', '\u{7f}')?;
    writeln!(out, "[{:8?}] [{:<8?}] [{:#?}]", "ab", 'x', 5u8)?;
    writeln!(out, "{:?} {:?} {:?}", (1, "two", 3.0f32), [1u8, 2, 3], ())?;
    writeln!(out, "{:?} {:?}", Some(Point { x: 1, y: -2 }), None::<Point>)?;
    writeln!(out, "{:?}", Ok::<u8, ()>(3))?;
    writeln!(out, "{:?} {:?}", Shape::Unit, Shape::Tuple(9, 'z'))?;
    writeln!(out, "{:?}", Shape::Named { origin: Point::default(), label: "home" })?;
    writeln!(out, "{:#?}", Shape::Named { origin: Point { x: 3, y: 4 }, label: "away" })?;
    writeln!(out, "{:#?}", (1, [Some(2), None]))?;
    writeln!(out, "{:?} {:x?} {:X?} {:02x?}", Wrapper(&[10, 11]), [10, 255], [10, 255], [1, 2])?;
    writeln!(out, "{} {:#} [{:>6}] [{:<6}] [{:^7.2}]", Hex(0xcafe), Hex(0xcafe), Padded("ab"), Padded("cd"), Padded("truncate"))?;
    writeln!(out, "{:?}", Listing)?;
    writeln!(out, "{:#?}", Listing)?;
    writeln!(out, "{}", Options)?;
    writeln!(out, "{:_>+09.3}", Options)?;
    writeln!(out, "{:p}", core::ptr::null::<u8>())?;
    writeln!(out, "{:?} {:?}", core::ptr::null::<u8>(), 0x1234usize as *const u8)?;
    writeln!(out, "{}", format_args!("{}-{}", 1, format_args!("{:03}", 2)))?;
    out.write_fmt(format_args!("{:>4}|{:<4}|\n", 'a', 'b'))?;
    out.write_char('c')?;
    out.write_str("har\n")?;
    writeln!(out, "{:?} {:?}", 1u8..4, ..=5i8)?;
    writeln!(out, "{:?} {:?}", core::cmp::Ordering::Less, core::num::NonZero::new(5u8))?;
    writeln!(out, "{:?}", "tab\there \u{1F600} \u{301} é \0")?;
    writeln!(out, "{}", i64::MAX as u64 + 1)?;
    Ok(())
}
