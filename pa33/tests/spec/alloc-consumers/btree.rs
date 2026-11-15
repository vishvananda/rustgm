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

use alloc::collections::btree_map::Entry;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Bound;

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

fn word_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in text.split(|c: char| !c.is_alphabetic()).filter(|w| !w.is_empty()) {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

fn run(out: &mut Out) -> fmt::Result {
    let mut map: BTreeMap<u32, &str> = BTreeMap::new();
    writeln!(out, "empty {} {:?} {:?}", map.is_empty(), map.get(&1), map.first_key_value())?;
    for (key, value) in [(5, "five"), (1, "one"), (9, "nine"), (3, "three"), (7, "seven")] {
        map.insert(key, value);
    }
    writeln!(out, "insert {:?} {:?} {}", map.insert(3, "THREE"), map.insert(4, "four"), map.len())?;
    writeln!(out, "map {:?}", map)?;
    writeln!(out, "get {:?} {:?} {} {}", map.get(&9), map.get(&2), map.contains_key(&7), map[&1])?;
    writeln!(out, "ends {:?} {:?}", map.first_key_value(), map.last_key_value())?;
    writeln!(out, "remove {:?} {:?} {:?}", map.remove(&5), map.remove(&5), map.remove_entry(&1))?;
    writeln!(out, "pop {:?} {:?} {:?}", map.pop_first(), map.pop_last(), map)?;
    if let Some(value) = map.get_mut(&7) {
        *value = "SEVEN";
    }
    map.entry(7).and_modify(|value| *value = "seven!").or_insert("unused");
    map.entry(8).or_insert_with(|| "eight");
    match map.entry(2) {
        Entry::Vacant(slot) => {
            slot.insert("two");
        }
        Entry::Occupied(_) => {}
    }
    if let Entry::Occupied(slot) = map.entry(4) {
        let key = *slot.key();
        writeln!(out, "occupied {} {:?}", key, slot.remove())?;
    }
    writeln!(out, "entries {:?}", map)?;
    writeln!(out, "keys {:?} values {:?}", map.keys().collect::<Vec<_>>(), map.values().rev().collect::<Vec<_>>())?;

    let squares: BTreeMap<i32, i32> = (-5..=5).map(|v| (v, v * v)).collect();
    writeln!(out, "range {:?}", squares.range(-2..2).collect::<Vec<_>>())?;
    writeln!(out, "range {:?}", squares.range(3..).map(|(k, _)| *k).collect::<Vec<_>>())?;
    writeln!(out, "range {:?}", squares.range((Bound::Excluded(-5), Bound::Included(-3))).map(|(_, v)| *v).collect::<Vec<_>>())?;
    writeln!(out, "bounds {:?} {:?}", squares.range(..0).next_back(), squares.range(1..).next())?;
    let mut halves = squares.clone();
    let upper = halves.split_off(&1);
    writeln!(out, "split_off {} {} {:?} {:?}", halves.len(), upper.len(), halves.last_key_value(), upper.first_key_value())?;
    halves.retain(|key, value| key % 2 == 0 && *value > 0);
    halves.extend(upper.iter().filter(|(k, _)| **k > 3).map(|(k, v)| (*k, -*v)));
    let mut other = BTreeMap::from([(100, 1), (-4, 0)]);
    halves.append(&mut other);
    for value in halves.values_mut() {
        *value += 1;
    }
    writeln!(out, "retain {:?} {}", halves, other.is_empty())?;
    writeln!(out, "sum {} max {:?} cmp {}", halves.values().sum::<i32>(), halves.iter().max_by_key(|(_, v)| **v), squares < halves)?;
    let into: Vec<(i32, i32)> = halves.into_iter().rev().take(2).collect();
    writeln!(out, "into_iter {:?}", into)?;

    let text = "the quick brown fox jumps over the lazy dog. The DOG barks; the fox runs!";
    let counts = word_counts(text);
    let mut by_count: Vec<(&String, &usize)> = counts.iter().collect();
    by_count.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    writeln!(out, "words {} top {:?}", counts.len(), &by_count[..3])?;
    let mut index: BTreeMap<char, Vec<&str>> = BTreeMap::new();
    for word in counts.keys() {
        index.entry(word.chars().next().unwrap_or('?')).or_default().push(word);
    }
    writeln!(out, "index {:?}", index.range('d'..='j').collect::<Vec<_>>())?;
    let mut nested: BTreeMap<&str, BTreeMap<u8, bool>> = BTreeMap::new();
    nested.entry("a").or_default().insert(1, true);
    nested.entry("b").or_default().insert(2, false);
    nested.entry("a").or_default().insert(0, false);
    writeln!(out, "nested {:?}", nested)?;
    writeln!(out, "{:#?}", nested.get("a"))?;

    let mut large = BTreeMap::new();
    let mut lcg = Lcg(7);
    let mut inserted = 0;
    for _ in 0..4000 {
        if large.insert(lcg.next() % 3000, inserted).is_none() {
            inserted += 1;
        }
    }
    let ordered = large.keys().zip(large.keys().skip(1)).all(|(a, b)| a < b);
    writeln!(out, "large {} {} {} {:?} {:?}", large.len(), inserted, ordered, large.first_key_value(), large.last_key_value())?;
    let mut removed = 0;
    for _ in 0..6000 {
        if large.remove(&(lcg.next() % 3000)).is_some() {
            removed += 1;
        }
    }
    let checksum = large.iter().fold(0u64, |acc, (k, v)| acc.wrapping_mul(1_000_003).wrapping_add(u64::from(*k) * 7 + *v as u64));
    writeln!(out, "after removal {} {} {} {}", large.len(), removed, large.range(1000..2000).count(), checksum)?;
    while let Some((key, _)) = large.pop_first() {
        if key > 2500 {
            break;
        }
    }
    writeln!(out, "drained {} {:?}", large.len(), large.keys().next())?;

    let mut set: BTreeSet<i32> = [5, 1, 9, 3, 7, 3, 1].into_iter().collect();
    writeln!(out, "set {:?} {}", set, set.len())?;
    writeln!(out, "insert {} {} {:?}", set.insert(4), set.insert(5), set)?;
    writeln!(out, "take {:?} {} {} {:?} {:?}", set.take(&7), set.contains(&9), set.remove(&2), set.first(), set.last())?;
    let evens: BTreeSet<i32> = (0..10).filter(|v| v % 2 == 0).collect();
    writeln!(out, "union {:?}", set.union(&evens).collect::<Vec<_>>())?;
    writeln!(out, "intersection {:?}", set.intersection(&evens).collect::<Vec<_>>())?;
    writeln!(out, "difference {:?} {:?}", set.difference(&evens).collect::<Vec<_>>(), evens.difference(&set).collect::<Vec<_>>())?;
    writeln!(out, "symmetric {:?}", set.symmetric_difference(&evens).collect::<Vec<_>>())?;
    writeln!(out, "operators {:?} {:?}", &set & &evens, &(&set | &evens) - &BTreeSet::from([0, 1, 2, 3]))?;
    writeln!(
        out,
        "relations {} {} {} {}",
        BTreeSet::from([4]).is_subset(&set),
        set.is_superset(&evens),
        set.is_disjoint(&BTreeSet::from([100])),
        set.range(4..).count()
    )?;
    let mut strings: BTreeSet<String> = ["pear", "apple", "fig"].iter().map(|s| s.to_string()).collect();
    strings.insert("banana".to_string());
    writeln!(out, "strings {:?} {} {:?}", strings, strings.contains("fig"), strings.range::<str, _>((Bound::Included("b"), Bound::Excluded("g"))).collect::<Vec<_>>())?;
    let tuples: BTreeSet<(u8, char)> = [(2, 'a'), (1, 'z'), (2, 'A'), (1, 'a')].into();
    writeln!(out, "tuples {:?} {:?}", tuples, tuples.iter().next_back())?;
    let mut drained = BTreeSet::from([3, 1, 2]);
    writeln!(out, "pop {:?} {:?} {:?} {:?}", drained.pop_first(), drained.pop_last(), drained.pop_first(), drained.pop_first())?;
    Ok(())
}
