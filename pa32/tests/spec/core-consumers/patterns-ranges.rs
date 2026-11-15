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

use core::ops::{Bound, ControlFlow, Range, RangeBounds, RangeInclusive};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Token {
    Number(i64),
    Op(char),
    Open,
    Close,
    End,
}

struct Lexer<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Lexer<'_> {
    fn next(&mut self) -> Result<Token, usize> {
        while let Some(b' ') = self.bytes.get(self.position) {
            self.position += 1;
        }
        let Some(&byte) = self.bytes.get(self.position) else {
            return Ok(Token::End);
        };
        self.position += 1;
        Ok(match byte {
            b'0'..=b'9' => {
                let mut value = i64::from(byte - b'0');
                while let Some(digit @ b'0'..=b'9') = self.bytes.get(self.position).copied() {
                    value = value * 10 + i64::from(digit - b'0');
                    self.position += 1;
                }
                Token::Number(value)
            }
            b'+' | b'-' | b'*' | b'/' | b'%' => Token::Op(byte as char),
            b'(' => Token::Open,
            b')' => Token::Close,
            _ => return Err(self.position - 1),
        })
    }
}

struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl Parser<'_> {
    fn advance(&mut self) -> Result<(), usize> {
        self.current = self.lexer.next()?;
        Ok(())
    }

    fn primary(&mut self) -> Result<i64, usize> {
        match self.current {
            Token::Number(value) => {
                self.advance()?;
                Ok(value)
            }
            Token::Op('-') => {
                self.advance()?;
                Ok(-self.primary()?)
            }
            Token::Open => {
                self.advance()?;
                let value = self.expression(0)?;
                if self.current != Token::Close {
                    return Err(self.lexer.position);
                }
                self.advance()?;
                Ok(value)
            }
            _ => Err(self.lexer.position),
        }
    }

    fn expression(&mut self, minimum: u8) -> Result<i64, usize> {
        let mut left = self.primary()?;
        loop {
            let (operator, precedence) = match self.current {
                Token::Op(op @ ('+' | '-')) => (op, 1),
                Token::Op(op @ ('*' | '/' | '%')) => (op, 2),
                _ => break,
            };
            if precedence < minimum {
                break;
            }
            self.advance()?;
            let right = self.expression(precedence + 1)?;
            left = match operator {
                '+' => left + right,
                '-' => left - right,
                '*' => left * right,
                '/' if right != 0 => left / right,
                '%' if right != 0 => left % right,
                _ => return Err(self.lexer.position),
            };
        }
        Ok(left)
    }
}

fn evaluate(text: &str) -> Result<i64, usize> {
    let mut parser = Parser { lexer: Lexer { bytes: text.as_bytes(), position: 0 }, current: Token::End };
    parser.advance()?;
    let value = parser.expression(0)?;
    if parser.current == Token::End { Ok(value) } else { Err(parser.lexer.position) }
}

fn classify(value: i32) -> &'static str {
    match value {
        i32::MIN..=-1 => "negative",
        0 => "zero",
        1 | 2 | 3 => "small",
        n if n % 2 == 0 => "even",
        4..100 => "odd",
        _ => "large odd",
    }
}

fn describe(pair: (i32, Option<&str>)) -> &'static str {
    match pair {
        (0, None) => "origin",
        (x, Some(name)) if x < 0 && name.len() > 3 => "negative long",
        (_, Some("zero" | "nil")) => "zero name",
        (1..=9, Some(_)) => "digit",
        (_, None) => "anonymous",
        _ => "other",
    }
}

fn span<R: RangeBounds<usize>>(range: R, len: usize) -> (usize, usize) {
    let start = match range.start_bound() {
        Bound::Included(start) => *start,
        Bound::Excluded(start) => start + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(end) => end + 1,
        Bound::Excluded(end) => *end,
        Bound::Unbounded => len,
    };
    (start, end)
}

fn hanoi(disks: u32, moves: &mut u32) {
    if disks > 0 {
        hanoi(disks - 1, moves);
        *moves += 1;
        hanoi(disks - 1, moves);
    }
}

fn ackermann(m: u32, n: u32) -> u32 {
    match (m, n) {
        (0, n) => n + 1,
        (m, 0) => ackermann(m - 1, 1),
        (m, n) => ackermann(m - 1, ackermann(m, n - 1)),
    }
}

fn first_negative(values: &[i32]) -> Option<usize> {
    let found = values.iter().enumerate().try_for_each(|(index, value)| {
        if *value < 0 { ControlFlow::Break(index) } else { ControlFlow::Continue(()) }
    });
    match found {
        ControlFlow::Break(index) => Some(index),
        ControlFlow::Continue(()) => None,
    }
}

fn middle(values: &[u8]) -> Option<u8> {
    let (first, rest) = values.split_first()?;
    let (last, _) = rest.split_last()?;
    Some(first / 2 + last / 2)
}

fn run(out: &mut Out) -> fmt::Result {
    for text in ["1 + 2 * 3", "(1 + 2) * 3", "100 / 7 % 5 - -4", "2 * (3 + (4 - 1)) * 5", "7 / 0", "2 +", "4 $ 4", "((8))", "1 2"] {
        writeln!(out, "evaluate {:?} = {:?}", text, evaluate(text))?;
    }
    write!(out, "classify")?;
    for value in [-5, 0, 2, 4, 37, 100, 101] {
        write!(out, " {}", classify(value))?;
    }
    writeln!(out)?;
    write!(out, "describe")?;
    for pair in [(0, None), (-2, Some("minus")), (5, Some("nil")), (5, Some("five")), (50, None), (50, Some("x"))] {
        write!(out, " [{}]", describe(pair))?;
    }
    writeln!(out)?;

    let point = (3, -3);
    let message = match point {
        (x, y) if x == y => "diagonal",
        (x, y) if x == -y => "anti-diagonal",
        (0, _) | (_, 0) => "axis",
        _ => "plane",
    };
    let nested = Some(Ok::<(u8, [u8; 2]), ()>((1, [2, 3])));
    let Some(Ok((head, [_, tail]))) = nested else {
        return Err(fmt::Error);
    };
    writeln!(out, "{} {} {}", message, head, tail)?;
    let value = 42u8;
    let bucket = match value {
        low @ 0..=9 => u32::from(low),
        mid @ (10..=19 | 40..=49) => u32::from(mid) * 10,
        high => u32::from(high) * 100,
    };
    let reference = &Some(5);
    let doubled = if let Some(inner) = reference { inner * 2 } else { 0 };
    let chained = if let Some(n) = reference
        && *n > 3
    {
        "big"
    } else {
        "small"
    };
    writeln!(out, "{} {} {} {}", bucket, doubled, chained, matches!(value, 40..=50 if value % 2 == 0))?;

    let mut grid = [[0u8; 4]; 3];
    'rows: for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            if r * c > 3 {
                break 'rows;
            }
            if c == 3 {
                continue 'rows;
            }
            *cell = (r * 4 + c) as u8;
        }
    }
    writeln!(out, "grid {:?}", grid)?;
    let mut counter = 0;
    let found = loop {
        counter += 7;
        if counter % 5 == 3 {
            break counter;
        }
    };
    let labelled = 'outer: {
        for candidate in 2..50 {
            if candidate * candidate > 200 {
                break 'outer Some(candidate);
            }
        }
        None
    };
    let mut stack = [1, 2, 3, 4];
    let mut top = stack.len();
    let mut popped = 0;
    while let Some(index) = top.checked_sub(1) {
        popped = popped * 10 + stack[index];
        stack[index] = 0;
        top = index;
    }
    writeln!(out, "loops {} {:?} {} {:?}", found, labelled, popped, stack)?;

    let range: Range<u32> = 3..8;
    let inclusive: RangeInclusive<i8> = -2..=2;
    writeln!(out, "range {} {} {} {} {:?}", range.len(), range.contains(&7), range.contains(&8), range.is_empty(), range)?;
    writeln!(out, "inclusive {} {} {} {:?}", inclusive.start(), inclusive.end(), inclusive.contains(&2), inclusive.clone().rev().nth(1))?;
    writeln!(out, "spans {:?} {:?} {:?} {:?} {:?}", span(2..5, 10), span(..=3, 10), span(4.., 10), span(.., 10), span((Bound::Excluded(1), Bound::Included(4)), 10))?;
    writeln!(out, "sums {} {} {} {}", (1..=100).sum::<u32>(), (0..20).step_by(5).sum::<u32>(), (10..0).count(), ('a'..='z').count())?;
    writeln!(out, "bounds {:?} {:?} {} {}", (..5).end_bound(), (1..).start_bound(), (0..=u8::MAX).count(), (-3..3).map(|v: i32| v.abs()).sum::<i32>())?;
    let letters = ['a', 'b', 'c', 'd', 'e'];
    writeln!(out, "slices {:?} {:?} {:?} {:?}", &letters[1..3], &letters[..=1], &letters[3..], letters.get(2..9))?;

    let mut moves = 0;
    hanoi(10, &mut moves);
    writeln!(out, "recursion {} {} {}", moves, ackermann(2, 3), ackermann(3, 3))?;
    writeln!(out, "control flow {:?} {:?} {:?} {:?}", first_negative(&[4, 2, -1, -7]), first_negative(&[1]), middle(&[10, 0, 30]), middle(&[1]))?;
    let outcome: Result<i32, &str> = (|| {
        let a = evaluate("6 * 7").map_err(|_| "parse")?;
        let b = i32::try_from(a).map_err(|_| "range")?;
        if b > 40 { Err("too big") } else { Ok(b) }
    })();
    writeln!(out, "closure try {:?}", outcome)?;
    let tuple = ((1, 2.5), [3, 4], "five");
    let ((a, b), [c, d], e) = tuple;
    let Token::Number(n) = Token::Number(9) else { return Err(fmt::Error) };
    let (mut x, mut y) = (1, 2);
    (x, y) = (y, x + y);
    [x, y] = [y * 2, x * 2];
    writeln!(out, "destructure {} {} {} {} {} {} {} {}", a, b, c, d, e, n, x, y)?;
    Ok(())
}
