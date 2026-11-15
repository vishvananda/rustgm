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

use core::cmp::{Ordering, Reverse};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Card {
    rank: u8,
    suit: Suit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Version {
    major: u8,
    minor: u8,
    tag: &'static str,
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.major.cmp(&other.major).then_with(|| self.minor.cmp(&other.minor)))
    }
}

fn insertion_sort<T: PartialOrd>(values: &mut [T]) {
    for i in 1..values.len() {
        let mut j = i;
        while j > 0 && values[j - 1] > values[j] {
            values.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn sift_down<T, F: Fn(&T, &T) -> bool>(values: &mut [T], mut root: usize, end: usize, less: &F) {
    loop {
        let mut child = 2 * root + 1;
        if child >= end {
            return;
        }
        if child + 1 < end && less(&values[child], &values[child + 1]) {
            child += 1;
        }
        if !less(&values[root], &values[child]) {
            return;
        }
        values.swap(root, child);
        root = child;
    }
}

fn heap_sort<T, F: Fn(&T, &T) -> bool>(values: &mut [T], less: F) {
    let len = values.len();
    for start in (0..len / 2).rev() {
        sift_down(values, start, len, &less);
    }
    for end in (1..len).rev() {
        values.swap(0, end);
        sift_down(values, 0, end, &less);
    }
}

fn checksum(values: &[u32]) -> u64 {
    values.iter().enumerate().fold(0u64, |acc, (i, v)| acc.wrapping_mul(31).wrapping_add(u64::from(*v) ^ i as u64))
}

fn run(out: &mut Out) -> fmt::Result {
    let mut small = [5, 3, 9, 1, 7, 3, 8, 2];
    small.sort_unstable();
    writeln!(out, "sort_unstable {:?}", small)?;
    small.sort_unstable_by(|a, b| b.cmp(a));
    writeln!(out, "descending {:?}", small)?;
    small.sort_unstable_by_key(|v| (v % 3, *v));
    writeln!(out, "by key {:?}", small)?;
    small.sort_unstable_by_key(|v| Reverse(*v));
    writeln!(out, "reverse {:?} sorted {}", small, small.is_sorted_by(|a, b| a >= b))?;

    let mut random = [0u32; 300];
    let mut lcg = Lcg(2026);
    for slot in random.iter_mut() {
        *slot = lcg.next() % 1000;
    }
    writeln!(out, "random before {:016x} sorted {}", checksum(&random), random.is_sorted())?;
    let mut by_heap = random;
    let mut by_insertion = random;
    random.sort_unstable();
    heap_sort(&mut by_heap, |a, b| a < b);
    insertion_sort(&mut by_insertion);
    writeln!(out, "random after {:016x} sorted {}", checksum(&random), random.is_sorted())?;
    writeln!(out, "agree {} {}", random == by_heap, random == by_insertion)?;
    writeln!(out, "extremes {} {} {} median {}", random[0], random[1], random[299], random[150])?;
    let distinct = 1 + random.windows(2).filter(|w| w[0] != w[1]).count();
    writeln!(out, "distinct {}", distinct)?;
    writeln!(
        out,
        "search {:?} {:?} {}",
        random.binary_search(&random[77]).map(|i| random[i]),
        random.binary_search(&1000),
        random.partition_point(|v| *v < 500)
    )?;

    let mut big = [0u32; 2000];
    for slot in big.iter_mut() {
        *slot = lcg.next();
    }
    let before = big.iter().fold(0u32, |acc, v| acc ^ v);
    big.sort_unstable();
    writeln!(out, "big sorted {} xor preserved {} {:016x}", big.is_sorted(), before == big.iter().fold(0u32, |acc, v| acc ^ v), checksum(&big))?;
    let mut pattern = [0u16; 512];
    for (i, slot) in pattern.iter_mut().enumerate() {
        *slot = if i % 2 == 0 { i as u16 } else { 512 - i as u16 };
    }
    pattern.sort_unstable();
    writeln!(out, "pattern {} {:?} {:?}", pattern.is_sorted(), &pattern[..4], &pattern[508..])?;
    let mut descending = [0i32; 100];
    for (i, slot) in descending.iter_mut().enumerate() {
        *slot = 100 - i as i32;
    }
    descending.sort_unstable();
    writeln!(out, "descending input {} {} {}", descending[0], descending[99], descending.is_sorted())?;

    let mut select = [9, 1, 8, 2, 7, 3, 6, 4, 5];
    let (lower, median, upper) = select.select_nth_unstable(4);
    writeln!(out, "select {} {} {}", lower.iter().all(|v| *v < 5), median, upper.iter().all(|v| *v > 5))?;

    let mut words = ["pear", "Apple", "fig", "apple", "banana", "", "Fig"];
    words.sort_unstable();
    writeln!(out, "words {:?}", words)?;
    words.sort_unstable_by(|a, b| a.len().cmp(&b.len()).then(b.cmp(a)));
    writeln!(out, "by length {:?}", words)?;
    words.sort_unstable_by_key(|w| w.bytes().next().map(|b| b.to_ascii_lowercase()));
    writeln!(out, "first letters {:?}", words.map(|w| w.chars().next().unwrap_or('-')))?;

    let mut floats = [2.5f64, -1.0, f64::INFINITY, 0.0, -0.0, 1e-9, f64::NEG_INFINITY, 7.25];
    floats.sort_unstable_by(f64::total_cmp);
    writeln!(out, "floats {:?}", floats)?;
    floats.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap_or(Ordering::Equal));
    writeln!(out, "floats max {} min {}", floats[0], floats[7])?;

    let mut cards = [
        Card { rank: 12, suit: Suit::Hearts },
        Card { rank: 3, suit: Suit::Spades },
        Card { rank: 12, suit: Suit::Clubs },
        Card { rank: 1, suit: Suit::Diamonds },
        Card { rank: 3, suit: Suit::Clubs },
    ];
    cards.sort_unstable();
    for card in &cards {
        write!(out, "{}{:?} ", card.rank, card.suit)?;
    }
    writeln!(out)?;
    cards.sort_unstable_by_key(|card| (Reverse(card.suit), card.rank));
    writeln!(out, "top {:?} max {:?}", cards[0], cards.iter().max())?;
    writeln!(out, "suit order {:?} {:?} {}", Suit::Clubs.cmp(&Suit::Spades), Suit::Hearts.max(Suit::Diamonds), Suit::Spades > Suit::Hearts)?;

    let mut pairs = [(2, 'b'), (1, 'z'), (2, 'a'), (1, 'a')];
    pairs.sort_unstable();
    writeln!(out, "tuples {:?}", pairs)?;
    let mut versions = [
        Version { major: 1, minor: 4, tag: "d" },
        Version { major: 0, minor: 9, tag: "a" },
        Version { major: 1, minor: 0, tag: "b" },
        Version { major: 1, minor: 2, tag: "c" },
    ];
    insertion_sort(&mut versions);
    for version in &versions {
        write!(out, "{}.{}{} ", version.major, version.minor, version.tag)?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "cmp {:?} {:?} {:?} {:?}",
        1.cmp(&2),
        "b".cmp("a"),
        [1, 2].cmp(&[1, 2]),
        (1, "x").partial_cmp(&(1, "y"))
    )?;
    writeln!(
        out,
        "ordering {:?} {:?} {:?} {}",
        Ordering::Less.reverse(),
        Ordering::Equal.then(Ordering::Greater),
        Ordering::Greater.then(Ordering::Less),
        Ordering::Less.is_le()
    )?;
    writeln!(out, "minmax {} {} {} {}", core::cmp::min(3, 9), core::cmp::max("a", "b"), 15.clamp(0, 10), (-3).clamp(0, 10))?;
    writeln!(
        out,
        "by key {} {}",
        core::cmp::max_by_key(-7, 5, |v: &i32| v.abs()),
        core::cmp::min_by(2.5, 1.5, |a: &f64, b: &f64| a.total_cmp(b))
    )?;
    Ok(())
}
