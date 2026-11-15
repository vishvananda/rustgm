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

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(i64),
    Name(String),
    Symbol(char),
}

fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c.is_ascii_digit() {
            let mut value = 0i64;
            while let Some(digit) = chars.peek().and_then(|d| d.to_digit(10)) {
                value = value * 10 + i64::from(digit);
                chars.next();
            }
            tokens.push(Token::Number(value));
        } else if c.is_alphabetic() || c == '_' {
            let mut name = String::new();
            while let Some(&letter) = chars.peek().filter(|l| l.is_alphanumeric() || **l == '_') {
                name.push(letter);
                chars.next();
            }
            tokens.push(Token::Name(name));
        } else {
            tokens.push(Token::Symbol(c));
            chars.next();
        }
    }
    tokens
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Push(i64),
    Load(usize),
    Store(usize),
    Add,
    Mul,
    Sub,
    JumpIfZero(usize),
    Jump(usize),
    Print,
    Halt,
}

fn execute(program: &[Op], output: &mut Vec<i64>) -> Result<usize, String> {
    let mut stack: Vec<i64> = Vec::new();
    let mut variables = [0i64; 4];
    let mut counter = 0;
    let mut steps = 0;
    loop {
        let op = *program.get(counter).ok_or_else(|| alloc::format!("pc {} out of range", counter))?;
        counter += 1;
        steps += 1;
        match op {
            Op::Push(value) => stack.push(value),
            Op::Load(slot) => stack.push(variables[slot]),
            Op::Store(slot) => variables[slot] = stack.pop().ok_or("stack underflow")?,
            Op::Add | Op::Mul | Op::Sub => {
                let right = stack.pop().ok_or("stack underflow")?;
                let left = stack.pop().ok_or("stack underflow")?;
                stack.push(match op {
                    Op::Add => left + right,
                    Op::Mul => left * right,
                    _ => left - right,
                });
            }
            Op::JumpIfZero(target) => {
                if stack.pop().ok_or("stack underflow")? == 0 {
                    counter = target;
                }
            }
            Op::Jump(target) => counter = target,
            Op::Print => output.push(stack.pop().ok_or("stack underflow")?),
            Op::Halt => return Ok(steps),
        }
    }
}

fn permutations(items: &mut Vec<char>, size: usize, found: &mut Vec<String>) {
    if size <= 1 {
        found.push(items.iter().collect());
        return;
    }
    for index in 0..size {
        permutations(items, size - 1, found);
        let swap = if size % 2 == 0 { index } else { 0 };
        items.swap(swap, size - 1);
    }
}

fn topological(dependencies: &BTreeMap<&str, Vec<&str>>) -> Result<Vec<String>, String> {
    let mut incoming: BTreeMap<&str, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (item, needs) in dependencies {
        incoming.entry(item).or_insert(0);
        for need in needs {
            *incoming.entry(item).or_insert(0) += 1;
            incoming.entry(need).or_insert(0);
            dependents.entry(need).or_default().push(item);
        }
    }
    let mut ready: VecDeque<&str> = incoming.iter().filter(|(_, count)| **count == 0).map(|(item, _)| *item).collect();
    let mut order = Vec::new();
    while let Some(item) = ready.pop_front() {
        order.push(item.to_string());
        for dependent in dependents.get(item).into_iter().flatten() {
            let count = incoming.get_mut(dependent).ok_or("missing node")?;
            *count -= 1;
            if *count == 0 {
                ready.push_back(dependent);
            }
        }
    }
    if order.len() == incoming.len() { Ok(order) } else { Err(alloc::format!("cycle after {:?}", order)) }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b_chars.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = alloc::vec![i + 1];
        for (j, cb) in b_chars.iter().enumerate() {
            let substitute = previous[j] + usize::from(ca != *cb);
            current.push(substitute.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[b_chars.len()]
}

fn big_factorial(n: u32) -> String {
    let mut digits: Vec<u32> = alloc::vec![1];
    for factor in 2..=n {
        let mut carry = 0u64;
        for digit in digits.iter_mut() {
            let value = u64::from(*digit) * u64::from(factor) + carry;
            *digit = (value % 1_000_000_000) as u32;
            carry = value / 1_000_000_000;
        }
        while carry > 0 {
            digits.push((carry % 1_000_000_000) as u32);
            carry /= 1_000_000_000;
        }
    }
    let mut text = digits.last().map_or(String::new(), |d| d.to_string());
    for digit in digits.iter().rev().skip(1) {
        text += &alloc::format!("{:09}", digit);
    }
    text
}

fn run(out: &mut Out) -> fmt::Result {
    let tokens = tokenize("let total_1 = (price * 12) + tax2; // done");
    writeln!(out, "tokens {} {:?}", tokens.len(), &tokens[..5])?;
    let names: BTreeSet<&str> = tokens.iter().filter_map(|t| if let Token::Name(n) = t { Some(n.as_str()) } else { None }).collect();
    let symbols: String = tokens.iter().filter_map(|t| if let Token::Symbol(c) = t { Some(*c) } else { None }).collect();
    writeln!(out, "names {:?} symbols {:?}", names, symbols)?;

    let program = [
        Op::Push(10),
        Op::Store(0),
        Op::Push(1),
        Op::Store(1),
        Op::Load(0),
        Op::JumpIfZero(17),
        Op::Load(1),
        Op::Load(0),
        Op::Mul,
        Op::Store(1),
        Op::Load(1),
        Op::Print,
        Op::Load(0),
        Op::Push(1),
        Op::Sub,
        Op::Store(0),
        Op::Jump(4),
        Op::Halt,
    ];
    let mut printed = Vec::new();
    writeln!(out, "vm {:?} {:?}", execute(&program, &mut printed), printed)?;
    writeln!(out, "vm errors {:?} {:?}", execute(&[Op::Add], &mut printed), execute(&[Op::Jump(9)], &mut printed))?;

    let mut found = Vec::new();
    permutations(&mut alloc::vec!['a', 'b', 'c', 'd'], 4, &mut found);
    found.sort();
    found.dedup();
    writeln!(out, "permutations {} {:?} {:?}", found.len(), &found[..3], found.last())?;

    let mut dependencies = BTreeMap::new();
    dependencies.insert("std", alloc::vec!["alloc", "core", "libc"]);
    dependencies.insert("alloc", alloc::vec!["core"]);
    dependencies.insert("rustgm", alloc::vec!["std", "unicode"]);
    dependencies.insert("unicode", alloc::vec!["core"]);
    writeln!(out, "order {:?}", topological(&dependencies))?;
    dependencies.insert("core", alloc::vec!["rustgm"]);
    writeln!(out, "cycle {:?}", topological(&dependencies))?;

    writeln!(out, "distance {} {} {} {}", edit_distance("kitten", "sitting"), edit_distance("", "abc"), edit_distance("straße", "strasse"), edit_distance("same", "same"))?;
    let factorial = big_factorial(60);
    writeln!(out, "factorial {} {}", factorial.len(), factorial)?;
    let digit_sum: u32 = big_factorial(200).bytes().map(|b| u32::from(b - b'0')).sum();
    writeln!(out, "digit sum {}", digit_sum)?;

    let mut memo: BTreeMap<u64, u128> = BTreeMap::new();
    fn fibonacci(n: u64, memo: &mut BTreeMap<u64, u128>) -> u128 {
        if n < 2 {
            return u128::from(n);
        }
        if let Some(known) = memo.get(&n) {
            return *known;
        }
        let value = fibonacci(n - 1, memo) + fibonacci(n - 2, memo);
        memo.insert(n, value);
        value
    }
    writeln!(out, "fibonacci {} {}", fibonacci(150, &mut memo), memo.len())?;

    let text = "it was the best of times it was the worst of times it was the age of wisdom";
    let mut frequency: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (position, word) in text.split(' ').enumerate() {
        frequency.entry(word).or_default().push(position);
    }
    let mut ranked: Vec<(&str, usize)> = frequency.iter().map(|(word, positions)| (*word, positions.len())).collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    writeln!(out, "ranked {:?} {:?}", &ranked[..4], frequency.get("times"))?;
    let mut groups: BTreeMap<usize, BTreeSet<&str>> = BTreeMap::new();
    for word in frequency.keys() {
        groups.entry(word.len()).or_default().insert(word);
    }
    writeln!(out, "groups {:?}", groups)?;
    let encoded: Vec<(char, usize)> = "aaabccdddd".chars().fold(Vec::new(), |mut runs: Vec<(char, usize)>, c| {
        match runs.last_mut() {
            Some((last, count)) if *last == c => *count += 1,
            _ => runs.push((c, 1)),
        }
        runs
    });
    let decoded: String = encoded.iter().map(|(c, n)| c.to_string().repeat(*n)).collect();
    writeln!(out, "runs {:?} {}", encoded, decoded)?;
    let mut triangle: Vec<Vec<u64>> = alloc::vec![alloc::vec![1]];
    for row in 1..20 {
        let previous = &triangle[row - 1];
        let mut next = alloc::vec![1];
        next.extend(previous.windows(2).map(|pair| pair[0] + pair[1]));
        next.push(1);
        triangle.push(next);
    }
    writeln!(out, "pascal {:?} {}", triangle[10], triangle[19].iter().sum::<u64>())?;
    Ok(())
}
