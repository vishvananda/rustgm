#![no_std]
#![no_main]

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;
use core::fmt::{self, Write};

#[link(name = "c")]
unsafe extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn abort() -> !;
    fn malloc(size: usize) -> *mut u8;
    fn free(pointer: *mut u8);
    fn posix_memalign(out: *mut *mut u8, align: usize, size: usize) -> i32;
}

/// The global allocator: libc's heap plus a count of live blocks and bytes.
struct Heap {
    blocks: Cell<usize>,
    bytes: Cell<usize>,
}

// The consumers are single-threaded.
unsafe impl Sync for Heap {}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut pointer = core::ptr::null_mut();
        if layout.align() <= 16 {
            pointer = unsafe { malloc(layout.size()) };
        } else if unsafe { posix_memalign(&mut pointer, layout.align(), layout.size()) } != 0 {
            pointer = core::ptr::null_mut();
        }
        if !pointer.is_null() {
            self.blocks.set(self.blocks.get() + 1);
            self.bytes.set(self.bytes.get() + layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        self.blocks.set(self.blocks.get() - 1);
        self.bytes.set(self.bytes.get() - layout.size());
        unsafe { free(pointer) }
    }
}

#[global_allocator]
static HEAP: Heap = Heap { blocks: Cell::new(0), bytes: Cell::new(0) };

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
    let mut status = match run(&mut out) {
        Ok(()) => 0,
        Err(fmt::Error) => 1,
    };
    // Everything `run` owned is dropped by now, so a leak or a double free shows here.
    if writeln!(out, "heap: {} blocks, {} bytes live", HEAP.blocks.get(), HEAP.bytes.get()).is_err() {
        status = 1;
    }
    out.flush();
    status
}

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Record {
    key: u32,
    name: String,
}

fn primes(limit: usize) -> Vec<usize> {
    let mut sieve = vec![true; limit + 1];
    let mut found = Vec::new();
    for candidate in 2..=limit {
        if sieve[candidate] {
            found.push(candidate);
            let mut multiple = candidate * candidate;
            while multiple <= limit {
                sieve[multiple] = false;
                multiple += candidate;
            }
        }
    }
    found
}

fn run(out: &mut Out) -> fmt::Result {
    let mut values: Vec<i32> = Vec::new();
    writeln!(out, "empty {} {} {:?} {:?}", values.len(), values.is_empty(), values.pop(), values.first())?;
    for value in 1..=10 {
        values.push(value * value);
    }
    writeln!(out, "push {:?} {} {}", values, values.len(), values.capacity() >= values.len())?;
    writeln!(out, "pop {:?} {:?} {:?}", values.pop(), values.last(), values.get(20))?;
    values.insert(0, -1);
    values.insert(5, -5);
    writeln!(out, "insert {:?}", values)?;
    writeln!(out, "remove {} {} {:?}", values.remove(5), values.swap_remove(0), values)?;
    values.truncate(6);
    values.extend([7, 7, 7, 8, 8]);
    values.extend_from_slice(&[9, 7]);
    writeln!(out, "extend {:?}", values)?;
    values.dedup();
    writeln!(out, "dedup {:?}", values)?;
    values.retain(|v| v % 2 == 1);
    writeln!(out, "retain {:?}", values)?;
    let drained: Vec<i32> = values.drain(1..3).collect();
    writeln!(out, "drain {:?} {:?}", drained, values)?;
    let tail = values.split_off(2);
    writeln!(out, "split_off {:?} {:?}", values, tail)?;
    let mut joined = tail.clone();
    joined.append(&mut values);
    writeln!(out, "append {:?} {:?}", joined, values)?;
    joined.resize(8, 0);
    joined.resize_with(10, || -3);
    writeln!(out, "resize {:?}", joined)?;
    let replaced: Vec<i32> = joined.splice(1..4, [100, 200]).collect();
    writeln!(out, "splice {:?} {:?}", replaced, joined)?;
    joined.reverse();
    joined.rotate_left(2);
    joined.sort();
    writeln!(out, "sort {:?} {:?}", joined, joined.binary_search(&100))?;
    joined.sort_by(|a, b| b.cmp(a));
    joined.dedup_by_key(|v| *v / 100);
    writeln!(out, "sort_by {:?}", joined)?;
    joined.clear();
    writeln!(out, "clear {} {}", joined.len(), joined.capacity() > 0)?;
    joined.shrink_to_fit();
    writeln!(out, "shrink {}", joined.capacity())?;

    let mut reserved: Vec<u64> = Vec::with_capacity(100);
    let before = reserved.as_ptr();
    for value in 0..100 {
        reserved.push(value);
    }
    writeln!(out, "with_capacity {} {} {}", reserved.capacity(), before == reserved.as_ptr(), reserved.iter().sum::<u64>())?;
    reserved.reserve(1000);
    writeln!(out, "reserve {} {}", reserved.capacity() >= 1100, reserved[99])?;
    let mut grown = Vec::new();
    let mut moves = 0;
    let mut last = grown.as_ptr();
    for value in 0..5000u32 {
        grown.push(value);
        if grown.as_ptr() != last {
            moves += 1;
            last = grown.as_ptr();
        }
    }
    writeln!(out, "growth {} moves<=20 {} checksum {}", grown.len(), moves <= 20, grown.iter().fold(0u32, |acc, v| acc.wrapping_mul(31).wrapping_add(*v)))?;

    let squares: Vec<u32> = (1..=6).map(|v| v * v).collect();
    let evens: Vec<u32> = squares.iter().copied().filter(|v| v % 2 == 0).collect();
    let (small, large): (Vec<u32>, Vec<u32>) = squares.iter().partition(|v| **v < 10);
    writeln!(out, "collect {:?} {:?} {:?} {:?}", squares, evens, small, large)?;
    let (indices, doubled): (Vec<usize>, Vec<u32>) = squares.iter().enumerate().map(|(i, v)| (i, v * 2)).unzip();
    writeln!(out, "unzip {:?} {:?}", indices, doubled)?;
    let total: u32 = squares.into_iter().rev().take(3).sum();
    writeln!(out, "into_iter {}", total)?;
    let nested = vec![vec![1, 2], vec![], vec![3], vec![4, 5, 6]];
    let flat: Vec<i32> = nested.iter().flatten().copied().collect();
    writeln!(out, "nested {:?} {:?} {:?} {}", nested, flat, nested.concat(), nested.iter().map(Vec::len).max().unwrap_or(0))?;
    let words = vec!["delta", "alpha", "charlie", "bravo"];
    let mut sorted = words.clone();
    sorted.sort_by_key(|w| core::cmp::Reverse(w.len()));
    writeln!(out, "words {:?} {:?} {:?} {:?}", sorted, words.join("-"), words.concat().len(), [[1, 2], [3, 4]].join(&0))?;
    let mut stable = vec![(2, 'a'), (1, 'b'), (2, 'c'), (1, 'd'), (2, 'e'), (1, 'f')];
    stable.sort_by_key(|pair| pair.0);
    writeln!(out, "stable {:?}", stable)?;
    let mut records: Vec<Record> = ["pear", "apple", "fig", "apple"]
        .iter()
        .enumerate()
        .map(|(index, name)| Record { key: 10 - index as u32, name: name.to_string() })
        .collect();
    records.sort_by(|a, b| a.name.cmp(&b.name).then(a.key.cmp(&b.key)));
    for record in &records {
        write!(out, "{}:{} ", record.name, record.key)?;
    }
    writeln!(out)?;
    records.dedup_by(|a, b| a.name == b.name);
    let cloned = records.clone();
    records[0].name.push('!');
    writeln!(out, "records {} {:?} {:?} {}", records.len(), records.iter().max().map(|r| r.key), cloned[0], cloned == records)?;
    writeln!(out, "contains {} {:?}", cloned.contains(&Record { key: 6, name: "fig".to_string() }), cloned.iter().position(|r| r.name == "pear"))?;

    let boxed: Box<[u8]> = vec![3, 1, 2].into_boxed_slice();
    let mut back = boxed.into_vec();
    back.sort_unstable();
    let array: [u8; 3] = back.clone().try_into().unwrap_or([0; 3]);
    writeln!(out, "boxed {:?} {:?} {:?}", back, array, Vec::from(array) == back)?;
    let units = vec![(); 1000];
    writeln!(out, "zst {} {}", units.len(), units.capacity() == usize::MAX)?;
    let bytes = b"hello".to_vec();
    let from_str = Vec::from("hi");
    writeln!(out, "from {:?} {:?} {:?}", bytes, from_str, Vec::from([1.5f32, 2.5]))?;
    let table = primes(100);
    writeln!(out, "primes {} {:?} {:?}", table.len(), &table[..5], table.last())?;
    let mut matrix = vec![vec![0i64; 4]; 4];
    for (r, row) in matrix.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            *cell = (r as i64 + 1) * (c as i64 - 1);
        }
    }
    let transposed: Vec<Vec<i64>> = (0..4).map(|c| matrix.iter().map(|row| row[c]).collect()).collect();
    let product: Vec<Vec<i64>> = matrix
        .iter()
        .map(|row| (0..4).map(|c| row.iter().zip(&transposed).map(|(v, column)| v * column[c]).sum()).collect())
        .collect();
    writeln!(out, "matrix {:?}", product)?;
    writeln!(out, "compare {} {} {:?}", vec![1, 2] < vec![1, 3], vec![1, 2] == [1, 2], vec![2].cmp(&vec![1, 9]))?;
    let mut windows = vec![1, 5, 2, 8, 3];
    windows.iter_mut().for_each(|v| *v *= 2);
    let peaks: Vec<i32> = windows.windows(3).filter(|w| w[1] > w[0] && w[1] > w[2]).map(|w| w[1]).collect();
    writeln!(out, "windows {:?} {:?} {:?}", peaks, windows.chunks(2).map(|c| c.iter().sum::<i32>()).collect::<Vec<_>>(), windows.split_first())?;
    let leaked: &'static mut [u16] = Vec::leak(vec![7u16, 8]);
    leaked[0] += 1;
    writeln!(out, "leak {:?}", leaked)?;
    Ok(())
}
