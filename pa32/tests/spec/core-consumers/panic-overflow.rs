// Negative control: arithmetic overflow is checked, panics and the handler aborts.
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

fn accumulate(values: &[u8]) -> u8 {
    let mut total = 0u8;
    for value in values {
        total += *value;
    }
    total
}

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "sum {}", accumulate(&[100, 100, 55]))?;
    writeln!(out, "wrapping {}", [100u8, 100, 56].iter().fold(0u8, |acc, v| acc.wrapping_add(*v)))?;
    writeln!(out, "before the overflow")?;
    out.flush();
    let total = accumulate(core::hint::black_box(&[100, 100, 56]));
    writeln!(out, "unreachable {}", total)?;
    Ok(())
}
