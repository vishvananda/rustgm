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

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(char::is_whitespace) {
        Cow::Owned(input.split_whitespace().collect::<Vec<_>>().join("_"))
    } else {
        Cow::Borrowed(input)
    }
}

fn title_case(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut start = true;
    for c in text.chars() {
        if start {
            result.extend(c.to_uppercase());
        } else {
            result.extend(c.to_lowercase());
        }
        start = !c.is_alphanumeric();
    }
    result
}

fn run_length(text: &str) -> String {
    let mut encoded = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        let mut count = 1;
        while chars.next_if_eq(&c).is_some() {
            count += 1;
        }
        if count > 1 {
            encoded += &count.to_string();
        }
        encoded.push(c);
    }
    encoded
}

fn run(out: &mut Out) -> fmt::Result {
    let mut text = String::new();
    writeln!(out, "empty {:?} {} {}", text, text.len(), text.is_empty())?;
    text.push_str("hello");
    text.push(',');
    text += " world";
    text.insert(0, '>');
    text.insert_str(1, "> ");
    writeln!(out, "build {:?} {} {}", text, text.len(), text.capacity() >= text.len())?;
    writeln!(out, "pop {:?} {:?} {:?}", text.pop(), text.remove(0), text)?;
    text.truncate(8);
    text.retain(|c| c != 'l');
    writeln!(out, "retain {:?}", text)?;
    text.replace_range(1..3, "[mid]");
    let drained: String = text.drain(..2).collect();
    writeln!(out, "drain {:?} {:?}", drained, text)?;
    text.clear();
    writeln!(out, "clear {:?} {}", text, text.capacity() > 0)?;

    let greeting = String::from("Grüße, Jürgen ❤ straße");
    writeln!(out, "len {} chars {} upper {:?}", greeting.len(), greeting.chars().count(), greeting.to_uppercase())?;
    writeln!(out, "lower {:?} ascii {:?}", "ÀÉÎ Σ ΌΣΟΣ İ".to_lowercase(), greeting.to_ascii_uppercase())?;
    writeln!(out, "replace {:?} {:?} {:?}", greeting.replace("ü", "ue"), greeting.replacen('r', "R", 1), "aaa".replace("a", ""))?;
    writeln!(out, "repeat {:?} {:?}", "ab".repeat(3), "-".repeat(0))?;
    let reversed: String = greeting.chars().rev().collect();
    writeln!(out, "reversed {:?}", reversed)?;
    let words: Vec<&str> = greeting.split(' ').collect();
    let owned: Vec<String> = words.iter().map(|w| w.to_string()).collect();
    writeln!(out, "split {:?} {:?} {:?}", words, owned.join("+"), owned.concat().len())?;
    let initials: String = words.iter().filter_map(|w| w.chars().next()).collect();
    writeln!(out, "initials {:?} {:?}", initials, title_case("the qUICK brown-fox ßtraße"))?;
    writeln!(out, "normalize {:?} {:?}", normalize("a  b\tc"), normalize("plain"))?;
    let mut cow = normalize("x y");
    cow.to_mut().push('!');
    let borrowed: Cow<'_, str> = Cow::from("lit");
    writeln!(out, "cow {} {} {} {}", cow, matches!(borrowed, Cow::Borrowed(_)), borrowed.clone().into_owned(), cow.len())?;
    writeln!(out, "rle {:?} {:?} {:?}", run_length("aaabccddddde"), run_length(""), run_length("ééé!"))?;

    writeln!(out, "to_string {} {} {} {} {}", 42.to_string(), (-7i64).to_string(), 2.5f32.to_string(), 'c'.to_string(), true.to_string())?;
    writeln!(out, "to_string {} {} {}", u128::MAX.to_string().len(), "str".to_string(), 1e-7f64.to_string())?;
    writeln!(out, "parse {:?} {:?} {:?}", "12".parse::<u8>().map(|v| v.to_string() + "!"), "1.50".parse::<f64>().map(|v| v.to_string()), "x".parse::<u8>().map_err(|e| e.to_string()))?;
    writeln!(out, "utf8 {:?} {:?}", String::from_utf8(alloc::vec![0xe2, 0x9d, 0xa4]), String::from_utf8(alloc::vec![0xff, b'a']).map_err(|e| e.utf8_error().valid_up_to()))?;
    writeln!(out, "lossy {:?} {:?}", String::from_utf8_lossy(b"ok\xff\xfe!"), matches!(String::from_utf8_lossy(b"fine"), Cow::Borrowed("fine")))?;
    writeln!(out, "utf16 {:?} {:?} {:?}", String::from_utf16(&[0x48, 0xd83d, 0xde00]), String::from_utf16(&[0xd800]).is_err(), String::from_utf16_lossy(&[0x61, 0xdc00]))?;
    let bytes = greeting.clone().into_bytes();
    writeln!(out, "bytes {} {:?} {:?}", bytes.len(), &bytes[..4], "héllo".bytes().map(|b| b as u32).sum::<u32>())?;
    let boxed: Box<str> = greeting.clone().into_boxed_str();
    let again: String = boxed.clone().into_string();
    writeln!(out, "boxed {} {} {}", boxed.len(), again == greeting, &*boxed == greeting.as_str())?;
    let owned: String = "slice".to_owned();
    let from_chars = String::from_iter(['a', 'b', 'c']);
    let from_strs: String = ["x", "y", "z"].into_iter().collect();
    let from_strings: String = alloc::vec![String::from("1"), String::from("2")].into_iter().collect();
    writeln!(out, "from {} {} {} {} {}", owned, from_chars, from_strs, from_strings, String::from('q'))?;
    let mut extended = String::from("0");
    extended.extend(['1', '2']);
    extended.extend(["34", "56"]);
    extended.extend(alloc::vec![String::from("7")]);
    extended.extend(&['8', '9']);
    writeln!(out, "extend {}", extended)?;
    let mut written = String::new();
    write!(written, "{}-{:03}-{:?}", "w", 7, Some("x"))?;
    writeln!(written, " {:>5}|", 1.5)?;
    written.write_char('✓')?;
    writeln!(out, "write {:?}", written)?;
    writeln!(out, "compare {} {} {} {:?}", owned == "slice", "slice" == owned, owned < String::from("slide"), owned.cmp(&"sl".to_string()))?;
    writeln!(out, "concat {}", String::from("a") + "b" + &String::from("c") + &format!("{}", 'd'))?;
    let mut lines = String::new();
    for (number, line) in "one\ntwo\n\nfour".lines().enumerate() {
        lines.push_str(&format!("{:>2}: {}\n", number + 1, line));
    }
    write!(out, "{}", lines)?;
    writeln!(out, "find {:?} {:?} {:?}", greeting.find("Jürgen"), greeting.char_indices().nth(3), greeting.split_once(", ").map(|(a, b)| (a.len(), b.len())))?;
    writeln!(out, "trim {:?} {:?}", "  x  ".trim().to_owned(), String::from("xxabxx").trim_matches('x'))?;
    let mut capacity = String::with_capacity(64);
    capacity.push('a');
    let reserved = capacity.capacity();
    capacity.shrink_to_fit();
    writeln!(out, "capacity {} {} {}", reserved >= 64, capacity.capacity() < reserved, capacity)?;
    let mut big = String::new();
    for index in 0..2000 {
        big.push(char::from(b'a' + (index % 26) as u8));
    }
    writeln!(out, "big {} {:?} {}", big.len(), &big[1990..], big.matches("xyz").count())?;
    let split: Vec<String> = big.as_bytes().chunks(700).map(|c| String::from_utf8_lossy(c).into_owned()).collect();
    writeln!(out, "chunks {:?} {}", split.iter().map(String::len).collect::<Vec<_>>(), split.concat() == big)?;
    let escaped: String = "tab\t\"quote\" é\n".escape_debug().collect();
    writeln!(out, "escape {} {}", escaped, "é".escape_unicode().to_string())?;
    Ok(())
}
