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

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::format;
use alloc::rc::Rc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Default)]
struct Config {
    name: String,
    retries: u8,
    tags: Vec<&'static str>,
    limits: BTreeMap<String, f32>,
    parent: Option<Box<Config>>,
}

#[derive(Debug)]
#[allow(dead_code)]
enum Event {
    Start,
    Data(Vec<u8>),
    Error { code: i32, message: String },
}

struct Money {
    cents: i64,
}

impl Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = format!("{}${}.{:02}", if self.cents < 0 { "-" } else { "" }, self.cents.abs() / 100, self.cents.abs() % 100);
        f.pad(&text)
    }
}

struct Matrix(Vec<Vec<i32>>);

impl Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let width = self.0.iter().flatten().map(|v| v.to_string().len()).max().unwrap_or(1);
        for row in &self.0 {
            let cells: Vec<String> = row.iter().map(|v| format!("{:>width$}", v)).collect();
            writeln!(f, "[{}]", cells.join(" "))?;
        }
        Ok(())
    }
}

fn show<T: Display + ?Sized>(value: &T) -> String {
    format!("<{}>", value)
}

fn debug_all(items: &[&dyn Debug]) -> String {
    items.iter().map(|item| format!("{:?}", item)).collect::<Vec<_>>().join(" | ")
}

fn table(rows: &[(&str, f64, u32)]) -> String {
    let mut text = format!("{:<8}|{:>9}|{:>5}\n", "name", "price", "qty");
    text += &format!("{:-<8}+{:->9}+{:->5}\n", "", "", "");
    for (name, price, quantity) in rows {
        text += &format!("{:<8}|{:>9.2}|{:>5}\n", name, price, quantity);
    }
    let total: f64 = rows.iter().map(|(_, price, quantity)| price * f64::from(*quantity)).sum();
    text + &format!("{:<8}|{:>9.2}|{:>5}\n", "total", total, rows.iter().map(|r| r.2).sum::<u32>())
}

fn run(out: &mut Out) -> fmt::Result {
    let plain = format!("{} {} {}", 1, "two", 3.5);
    let positional = format!("{1}{0}{1}", "a", "b");
    let name = "inline";
    let width = 9;
    let named = format!("{name:>width$}|{name:<width$}|{name:^width$}|{value:+.2}", value = 2.0f32);
    writeln!(out, "{:?} {:?} {:?}", plain, positional, named)?;
    writeln!(out, "{} {} {}", format!("{:05}", -42), format!("{:#x}/{:#o}/{:#b}", 255, 8, 5), format!("{:e}", 1234.5))?;
    writeln!(out, "{}", format!("{:?} {:?} {:?}", "quote\"d", 'c', String::from("line\n")))?;
    writeln!(out, "{} {}", String::new() + &format!("{}", u64::MAX) + &i128::MIN.to_string(), format!("{{}}").len())?;

    let numbers = alloc::vec![1, 2, 3];
    let strings = alloc::vec![String::from("a"), String::from("b c")];
    let pairs: Vec<(u8, Option<&str>)> = alloc::vec![(1, Some("x")), (2, None)];
    writeln!(out, "{:?} {:?} {:?} {:?}", numbers, strings, pairs, Vec::<u8>::new())?;
    writeln!(out, "{:?} {:?} {:?}", &numbers[1..], Box::new(5), Rc::new("rc"))?;
    writeln!(out, "{:#?}", pairs)?;
    let mut map = BTreeMap::new();
    map.insert("one", alloc::vec![1]);
    map.insert("two", alloc::vec![2, 2]);
    let set: BTreeSet<char> = "hello".chars().collect();
    let deque: VecDeque<i8> = [-1, 0, 1].into();
    writeln!(out, "{:?} {:?} {:?}", map, set, deque)?;
    writeln!(out, "{:#?}", map)?;
    writeln!(out, "{:02x?} {:X?} {:5?} {:<4?}|", alloc::vec![10, 255], alloc::vec![171], alloc::vec![1, 2], Some(1))?;

    let mut config = Config { name: "base".to_string(), retries: 3, tags: alloc::vec!["a", "b"], ..Config::default() };
    config.limits.insert("cpu".to_string(), 0.5);
    let child = Config { name: "child".to_string(), parent: Some(Box::new(config.clone())), ..Config::default() };
    writeln!(out, "{:?}", config)?;
    writeln!(out, "{:#?}", child)?;
    let events = [Event::Start, Event::Data(alloc::vec![1, 2]), Event::Error { code: -2, message: "bad".to_string() }];
    writeln!(out, "{:?}", events)?;
    writeln!(out, "{}", debug_all(&[&1u8, &"s", &Some(2.5), &config.tags, &(), &events[0]]))?;

    let prices = [Money { cents: 123456 }, Money { cents: -5 }, Money { cents: 0 }];
    for price in &prices {
        write!(out, "[{}] [{:>10}] [{:<10}] [{:^10}] ", price, price, price, price)?;
    }
    writeln!(out)?;
    writeln!(out, "{} {} {}", show("str"), show(&7), show(&prices[0]))?;
    let boxed: Vec<Box<dyn Display>> = alloc::vec![Box::new(1.5), Box::new("text"), Box::new('c'), Box::new(Money { cents: 250 })];
    let joined: Vec<String> = boxed.iter().map(|item| item.to_string()).collect();
    writeln!(out, "{}", joined.join(", "))?;
    write!(out, "{}", Matrix(alloc::vec![alloc::vec![1, -20, 300], alloc::vec![4000, 5, -6]]))?;
    write!(out, "{}", table(&[("widget", 2.5, 4), ("gadget", 10.125, 1), ("bolt", 0.05, 250)]))?;

    let cow: Cow<'_, str> = Cow::Owned(format!("{}-{}", "cow", 1));
    writeln!(out, "{} {:?} {:>8}|", cow, cow, Cow::Borrowed("b"))?;
    writeln!(out, "{}", alloc::fmt::format(format_args!("{}{}", "format_args", '!')))?;
    writeln!(out, "{:?} {:?}", format_args!("static").as_str(), format_args!("{}", 1).as_str())?;
    let mut buffer = String::new();
    for index in 0..5 {
        if index > 0 {
            buffer.push_str(", ");
        }
        write!(buffer, "{index}:{:>3}", index * index)?;
    }
    writeln!(out, "{}", buffer)?;
    let floats: Vec<String> = [0.1f64, 1.0 / 3.0, 1e21, 1e-7, -0.0, f64::INFINITY].iter().map(|v| v.to_string()).collect();
    let round_trip = floats.iter().map(|text| text.parse::<f64>().map(|v| v.to_string() == *text)).collect::<Vec<_>>();
    writeln!(out, "{:?} {:?}", floats, round_trip)?;
    writeln!(out, "{} {} {}", format!("{:.3}", 2.0f64 / 3.0), format!("{:10.3e}|", 123456.789), format!("{:+}", 7))?;
    let long = format!("{}", "x".repeat(300));
    let padded = format!("{:>400}", "y");
    writeln!(out, "{} {} {}", long.len(), padded.len(), padded.trim_start())?;
    let nested = format!("{}", format!("{}", format!("{:?}", Some(format!("{}", 1)))));
    writeln!(out, "{} {}", nested, format!("{:?}", (1, "a", [Some(1.5)], 'x', ())))?;
    let unicode = format!("{:^9}|{:>6}|{:<6}|", "日本", "é", "😀");
    writeln!(out, "{} {}", unicode, unicode.chars().count())?;
    Ok(())
}
