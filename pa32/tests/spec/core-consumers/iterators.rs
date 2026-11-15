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

struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0 as usize, Some(self.0 as usize))
    }
}

impl ExactSizeIterator for Countdown {}

fn total(values: impl Iterator<Item = u32>) -> u32 {
    values.sum()
}

fn dynamic(values: &mut dyn Iterator<Item = u32>) -> u32 {
    values.fold(0, |acc, v| acc * 10 + v)
}

fn run(out: &mut Out) -> fmt::Result {
    let values = [3i64, -1, 4, -1, 5, -9, 2, 6];
    writeln!(out, "sum {} product {}", values.iter().sum::<i64>(), values.iter().take(4).product::<i64>())?;
    writeln!(out, "min {:?} max {:?}", values.iter().min(), values.iter().max())?;
    writeln!(
        out,
        "min_by_key {:?} max_by {:?}",
        values.iter().min_by_key(|v| v.abs()),
        values.iter().max_by(|a, b| (*a % 4).cmp(&(*b % 4)))
    )?;
    writeln!(
        out,
        "count {} last {:?} nth {:?}",
        values.iter().filter(|v| **v < 0).count(),
        values.iter().last(),
        values.iter().nth(2)
    )?;
    writeln!(
        out,
        "any {} all {} find {:?}",
        values.iter().any(|v| *v > 5),
        values.iter().all(|v| *v != 0),
        values.iter().find(|v| **v % 2 == 0)
    )?;
    writeln!(
        out,
        "position {:?} find_map {:?}",
        values.iter().position(|v| *v == 5),
        values.iter().find_map(|v| u8::try_from(*v).ok().filter(|b| *b > 4))
    )?;
    writeln!(
        out,
        "fold {} try_fold {:?} {:?}",
        values.iter().fold(0i64, |acc, v| acc * 2 + v),
        values.iter().try_fold(0i8, |acc, v| acc.checked_add(*v as i8)),
        values.iter().try_fold(100i8, |acc, v| acc.checked_mul(*v as i8))
    )?;
    write!(out, "map filter enumerate")?;
    for (index, square) in values.iter().map(|v| v * v).enumerate().filter(|(i, _)| i % 3 == 0) {
        write!(out, " {}:{}", index, square)?;
    }
    writeln!(out)?;
    write!(out, "zip rev chain")?;
    for (a, b) in values.iter().zip(values.iter().rev()).take(3).chain([(&10, &20)]) {
        write!(out, " ({},{})", a, b)?;
    }
    writeln!(out)?;
    write!(out, "skip step_by")?;
    for v in values.iter().skip(1).step_by(2) {
        write!(out, " {}", v)?;
    }
    writeln!(out)?;
    write!(out, "take_while skip_while")?;
    for v in values.iter().take_while(|v| **v != 5).chain(values.iter().skip_while(|v| **v != -9)) {
        write!(out, " {}", v)?;
    }
    writeln!(out)?;
    write!(out, "scan")?;
    for v in values.iter().scan(0i64, |state, v| {
        *state += v;
        if *state > 10 { None } else { Some(*state) }
    }) {
        write!(out, " {}", v)?;
    }
    writeln!(out)?;
    write!(out, "flat_map")?;
    for v in (1..4u32).flat_map(|n| (0..n).map(move |k| n * 10 + k)) {
        write!(out, " {}", v)?;
    }
    writeln!(out)?;
    let nested = [[1u8, 2], [3, 4], [5, 6]];
    writeln!(out, "flatten {}", nested.iter().flatten().map(|v| u32::from(*v)).sum::<u32>())?;
    let mut peekable = values.iter().peekable();
    let mut runs = 0;
    while let Some(v) = peekable.next() {
        if let Some(next) = peekable.peek() {
            if (**next < 0) != (*v < 0) {
                runs += 1;
            }
        }
    }
    writeln!(out, "peekable sign changes {}", runs)?;
    write!(out, "cycle inspect")?;
    let mut seen = 0;
    for v in [1u8, 2, 3].iter().cycle().inspect(|_| seen += 1).take(7) {
        write!(out, " {}", v)?;
    }
    writeln!(out, " seen {}", seen)?;
    writeln!(
        out,
        "ranges {} {} {} {}",
        (1..=10).sum::<u32>(),
        (0..10).rev().step_by(3).count(),
        (5..5).count(),
        (b'a'..=b'e').map(|b| b as char).filter(|c| *c != 'c').count()
    )?;
    let mut counter = 0u32;
    let from_fn = core::iter::from_fn(|| {
        counter += 1;
        if counter <= 4 { Some(counter * counter) } else { None }
    });
    writeln!(out, "from_fn {}", total(from_fn))?;
    let powers = core::iter::successors(Some(1u32), |v| v.checked_mul(10));
    writeln!(out, "successors {} last {:?}", powers.clone().count(), powers.last())?;
    writeln!(
        out,
        "repeat once empty {} {} {}",
        total(core::iter::repeat(7).take(3)),
        total(core::iter::once(9)),
        total(core::iter::empty())
    )?;
    let mut countdown = Countdown(5);
    writeln!(out, "custom len {} first {:?}", countdown.len(), countdown.next())?;
    writeln!(out, "dyn {}", dynamic(&mut countdown))?;
    writeln!(out, "by_ref {}", {
        let mut iter = values.iter();
        let head: i64 = iter.by_ref().take(2).sum();
        head * 100 + iter.count() as i64
    })?;
    let (evens, odds): (i64, i64) = values.iter().fold((0, 0), |(e, o), v| {
        if v % 2 == 0 { (e + v, o) } else { (e, o + v) }
    });
    writeln!(out, "partition sums {} {}", evens, odds)?;
    writeln!(
        out,
        "cmp {:?} eq {} lt {}",
        values.iter().cmp(values.iter().skip(1)),
        values.iter().eq(values.iter()),
        [1, 2].iter().lt([1, 3].iter())
    )?;
    writeln!(out, "unzip-free max_by_key {:?}", values.iter().enumerate().max_by_key(|(_, v)| **v))?;
    writeln!(out, "double ended {:?} {:?}", values.iter().next_back(), values.iter().rfind(|v| **v < 0))?;
    writeln!(out, "is_sorted {} {}", values.is_sorted(), [1, 2, 2, 9].is_sorted())?;
    Ok(())
}
