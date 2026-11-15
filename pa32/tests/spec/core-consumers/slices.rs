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

fn run(out: &mut Out) -> fmt::Result {
    let mut data = [9u32, 4, 7, 1, 8, 2, 6, 3, 5, 0];
    writeln!(out, "len {} first {:?} last {:?}", data.len(), data.first(), data.last())?;
    writeln!(out, "get {:?} {:?} {:?}", data.get(2), data.get(10), data.get(3..6))?;
    let (left, right) = data.split_at(4);
    writeln!(out, "split_at {:?} | {:?}", left, right)?;
    writeln!(out, "contains {} {}", data.contains(&7), data.contains(&70))?;
    writeln!(out, "starts {} ends {}", data.starts_with(&[9, 4]), data.ends_with(&[5, 0, 1]))?;
    write!(out, "windows")?;
    for pair in data.windows(2).filter(|w| w[0] < w[1]) {
        write!(out, " {}<{}", pair[0], pair[1])?;
    }
    writeln!(out)?;
    write!(out, "chunks")?;
    for chunk in data.chunks(3) {
        write!(out, " {:?}", chunk)?;
    }
    writeln!(out)?;
    writeln!(out, "chunks_exact remainder {:?}", data.chunks_exact(4).remainder())?;
    write!(out, "rchunks")?;
    for chunk in data.rchunks(4) {
        write!(out, " {:?}", chunk)?;
    }
    writeln!(out)?;
    data.reverse();
    writeln!(out, "reverse {:?}", data)?;
    data.rotate_left(3);
    writeln!(out, "rotate_left {:?}", data)?;
    data.swap(0, 9);
    writeln!(out, "swap {:?}", data)?;
    data[2..5].fill(11);
    writeln!(out, "fill {:?}", data)?;
    let mut copy = [0u32; 4];
    copy.copy_from_slice(&data[6..]);
    writeln!(out, "copy_from_slice {:?}", copy)?;
    data.copy_within(0..3, 7);
    writeln!(out, "copy_within {:?}", data)?;
    let (a, b) = data.split_at_mut(5);
    a[0] += b[0];
    core::mem::swap(&mut a[1], &mut b[1]);
    writeln!(out, "split_at_mut {:?}", data)?;
    for value in data.iter_mut().skip(1).step_by(3) {
        *value *= 2;
    }
    writeln!(out, "iter_mut {:?}", data)?;

    let sorted = [1, 3, 5, 7, 9, 11, 13];
    writeln!(
        out,
        "binary_search {:?} {:?} {:?}",
        sorted.binary_search(&7),
        sorted.binary_search(&8),
        sorted.binary_search(&0)
    )?;
    writeln!(out, "partition_point {}", sorted.partition_point(|v| *v < 6))?;
    writeln!(
        out,
        "position {:?} rposition {:?}",
        sorted.iter().position(|v| *v > 4),
        sorted.iter().rposition(|v| *v < 4)
    )?;
    let text = b"alpha,beta,,gamma";
    write!(out, "split")?;
    for part in text.split(|b| *b == b',') {
        write!(out, " {}", part.len())?;
    }
    writeln!(out)?;
    write!(out, "splitn")?;
    for part in text.splitn(2, |b| *b == b',') {
        write!(out, " {}", part.len())?;
    }
    writeln!(out)?;
    let grid = [[1u8, 2, 3], [4, 5, 6]];
    writeln!(
        out,
        "nested {:?} flat {:?} sum {}",
        grid[1],
        grid.as_flattened(),
        grid.iter().flatten().map(|v| u32::from(*v)).sum::<u32>()
    )?;
    let [first, .., last] = sorted;
    writeln!(out, "pattern {} {}", first, last)?;
    match &sorted[..] {
        [a, b, rest @ ..] => writeln!(out, "rest {} {} {}", a, b, rest.len())?,
        _ => writeln!(out, "short")?,
    }
    if let [x, y] = &sorted[1..3] {
        writeln!(out, "pair {} {}", x, y)?;
    }
    if let Some((head, tail)) = sorted.split_first() {
        writeln!(out, "split_first {} {}", head, tail.len())?;
    }
    writeln!(out, "eq {} cmp {:?}", sorted[..2] == [1, 3], sorted[..2].cmp(&sorted[1..3]))?;
    writeln!(out, "rev max {:?}", sorted.iter().rev().take(3).copied().max())?;
    let empty: [u32; 0] = [];
    writeln!(out, "empty {} {:?} {:?}", empty.is_empty(), empty.first(), empty.iter().max())?;
    Ok(())
}
