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

fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

fn is_palindrome(text: &str) -> bool {
    let mut letters = text.chars().filter(|c| c.is_alphanumeric()).map(|c| c.to_ascii_lowercase());
    loop {
        match (letters.next(), letters.next_back()) {
            (Some(a), Some(b)) if a == b => continue,
            (Some(_), Some(_)) => return false,
            _ => return true,
        }
    }
}

fn caesar(text: &str, shift: u8, buffer: &mut [u8]) -> usize {
    let mut len = 0;
    for byte in text.bytes() {
        buffer[len] = match byte {
            b'a'..=b'z' => (byte - b'a' + shift) % 26 + b'a',
            b'A'..=b'Z' => (byte - b'A' + shift) % 26 + b'A',
            other => other,
        };
        len += 1;
    }
    len
}

fn run(out: &mut Out) -> fmt::Result {
    for c in ['a', 'Z', '7', ' ', '\n', 'é', 'ß', 'Σ', '中', '😀', '\u{301}', '٣'] {
        writeln!(
            out,
            "{:?} {} alpha={} num={} alnum={} ws={} upper={} lower={} ctrl={} ascii={} len={}",
            c,
            c as u32,
            c.is_alphabetic(),
            c.is_numeric(),
            c.is_alphanumeric(),
            c.is_whitespace(),
            c.is_uppercase(),
            c.is_lowercase(),
            c.is_control(),
            c.is_ascii(),
            c.len_utf8()
        )?;
    }
    write!(out, "case")?;
    for c in ['a', 'Z', 'ß', 'ǆ', 'İ', 'Σ', 'ﬁ', '1'] {
        write!(out, " {}>", c)?;
        for upper in c.to_uppercase() {
            write!(out, "{}", upper)?;
        }
        write!(out, "/")?;
        for lower in c.to_lowercase() {
            write!(out, "{}", lower)?;
        }
    }
    writeln!(out)?;
    writeln!(
        out,
        "ascii {} {} {} {} {} {}",
        'q'.to_ascii_uppercase(),
        'Q'.to_ascii_lowercase(),
        'x'.is_ascii_hexdigit(),
        'f'.is_ascii_hexdigit(),
        '!'.is_ascii_punctuation(),
        'é'.eq_ignore_ascii_case(&'É')
    )?;
    writeln!(out, "digit {:?} {:?} {:?} {:?}", '9'.to_digit(10), 'z'.to_digit(36), 'g'.to_digit(16), '٣'.to_digit(10))?;
    writeln!(out, "escape {} {} {} {}", '\n'.escape_default(), 'é'.escape_unicode(), '"'.escape_debug(), 'é'.escape_default())?;
    writeln!(out, "order {} {:?} {:?}", 'a' < 'b', char::MAX, ('a'..='e').rev().nth(1))?;

    let text = "The quick brown fox jumps over the lazy dog";
    writeln!(out, "len {} words {} chars {}", text.len(), count_words(text), text.chars().count())?;
    writeln!(out, "find {:?} {:?} {:?} {:?}", text.find("fox"), text.find('q'), text.rfind("the"), text.find("cat"))?;
    writeln!(out, "find {:?} {:?}", text.find(char::is_uppercase), text.find(|c: char| c == 'z' || c == 'y'))?;
    writeln!(
        out,
        "contains {} {} {} {}",
        text.contains("lazy dog"),
        text.contains("lazy cat"),
        text.contains('x'),
        text.contains(&['!', '?'][..])
    )?;
    writeln!(out, "affix {} {} {:?} {:?}", text.starts_with("The"), text.ends_with("cat"), text.strip_prefix("The "), text.strip_suffix("fox"))?;
    writeln!(out, "slice {:?} {:?} {:?}", &text[4..9], text.get(40..), text.get(40..99))?;
    write!(out, "split")?;
    for word in text.split(' ').filter(|w| w.len() > 4) {
        write!(out, " {}", word)?;
    }
    writeln!(out)?;
    write!(out, "rsplit")?;
    for word in text.rsplitn(3, ' ') {
        write!(out, " [{}]", word)?;
    }
    writeln!(out)?;
    writeln!(out, "split_once {:?} {:?} {:?}", "key=value=more".split_once('='), "key=value=more".rsplit_once('='), "novalue".split_once('='))?;
    write!(out, "csv")?;
    for field in "a,b,,d,".split(',') {
        write!(out, " <{}>", field)?;
    }
    for field in "a,b,,d,".split_terminator(',') {
        write!(out, " ({})", field)?;
    }
    writeln!(out)?;
    write!(out, "multi")?;
    for field in "one, two;three  four".split(&[',', ';', ' '][..]).filter(|f| !f.is_empty()) {
        write!(out, " {}", field)?;
    }
    for field in "aXXbXXc".split("XX") {
        write!(out, " {}", field)?;
    }
    for field in "1..2...3".split_inclusive('.') {
        write!(out, " {}", field)?;
    }
    writeln!(out)?;
    write!(out, "lines")?;
    for (number, line) in "first\nsecond\r\n\nfourth".lines().enumerate() {
        write!(out, " {}:{:?}", number, line)?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "trim {:?} {:?} {:?} {:?}",
        "  padded\t\n".trim(),
        "  padded  ".trim_start(),
        "xxhixx".trim_matches('x'),
        "007".trim_start_matches('0')
    )?;
    writeln!(out, "matches {} {:?}", "banana".matches("an").count(), "banana".match_indices('a').map(|(i, _)| i).last())?;
    writeln!(out, "rmatch {:?}", "abcabc".rmatch_indices("bc").next())?;
    writeln!(out, "palindrome {} {} {}", is_palindrome("A man, a plan, a canal: Panama"), is_palindrome("Not one"), is_palindrome(""))?;
    writeln!(out, "cmp {:?} {} {} {}", "apple".cmp("apricot"), "a" < "B", "abc" == "abc", "Hello".eq_ignore_ascii_case("hELLO"))?;
    writeln!(out, "parse {:?} {:?} {:?}", "  42 ".trim().parse::<i32>(), "x".parse::<i32>().is_err(), "-7".parse::<i8>())?;

    let unicode = "héllo wörld ∑ 日本語 😀";
    writeln!(out, "unicode len {} chars {} words {}", unicode.len(), unicode.chars().count(), count_words(unicode))?;
    write!(out, "indices")?;
    for (index, c) in unicode.char_indices().filter(|(_, c)| !c.is_ascii()) {
        write!(out, " {}:{}", index, c)?;
    }
    writeln!(out)?;
    writeln!(out, "boundary {} {} {:?} {:?}", unicode.is_char_boundary(1), unicode.is_char_boundary(2), unicode.get(1..2), unicode.get(1..3))?;
    write!(out, "rev ")?;
    for c in unicode.chars().rev() {
        write!(out, "{}", c)?;
    }
    writeln!(out)?;
    writeln!(out, "bytes {:?} {:x?}", unicode.bytes().filter(|b| *b >= 0x80).count(), "é∑😀".as_bytes())?;
    writeln!(out, "find {:?} {:?} {}", unicode.find('∑'), unicode.find("日本"), unicode.contains("wörld ∑ 日"))?;
    writeln!(out, "utf16 {} {:?}", unicode.encode_utf16().count(), "😀".encode_utf16().next())?;
    writeln!(out, "escape {} | {} | {}", "tab\t\"q\" é".escape_debug(), "é\n".escape_default(), "ab".escape_unicode())?;
    writeln!(out, "ascii {} {} {:?}", unicode.is_ascii(), text.is_ascii(), "abc".as_bytes().first().map(u8::is_ascii_lowercase))?;
    let needle = "needle in a rather long haystack with a needle near the end";
    writeln!(out, "long {:?} {:?} {}", needle.find("needle near"), needle.rfind("needle"), needle.matches("ee").count())?;

    let mut buffer = [0u8; 64];
    let len = caesar("Hello, World!", 3, &mut buffer);
    let shifted = core::str::from_utf8(&buffer[..len]).map_err(|_| fmt::Error)?;
    writeln!(out, "caesar {}", shifted)?;
    let mutable = core::str::from_utf8_mut(&mut buffer[..len]).map_err(|_| fmt::Error)?;
    mutable.make_ascii_uppercase();
    writeln!(out, "upper {} {:?}", mutable, mutable.split_at(5))?;
    write!(out, "lossy")?;
    for chunk in b"ok\xffbad\xe2\x82tail".utf8_chunks() {
        write!(out, " {:?}+{}", chunk.valid(), chunk.invalid().len())?;
    }
    writeln!(out)?;
    writeln!(out, "empty {} {:?} {:?} {}", "".is_empty(), "".chars().next(), "".split(',').count(), "".lines().count())?;
    writeln!(out, "repeat-free {:?} {:?}", "abc".char_indices().nth(2), "ab".chars().zip("xyz".chars()).last())?;
    writeln!(out, "bytes eq {} {}", b"abc" == "abc".as_bytes(), "abc".bytes().rev().eq(b"cba".iter().copied()))?;
    Ok(())
}
