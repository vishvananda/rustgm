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

use core::cell::Cell;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Index, IndexMut, Mul, Neg, Not, Sub};

trait Shape {
    fn area(&self) -> u32;
    fn name(&self) -> &'static str {
        "shape"
    }
    fn describe(&self, out: &mut dyn Write) -> fmt::Result {
        write!(out, "{}:{}", self.name(), self.area())
    }
}

struct Square(u32);
struct Rect {
    w: u32,
    h: u32,
}

impl Shape for Square {
    fn area(&self) -> u32 {
        self.0 * self.0
    }
    fn name(&self) -> &'static str {
        "square"
    }
}

impl Shape for Rect {
    fn area(&self) -> u32 {
        self.w * self.h
    }
}

fn total_area(shapes: &[&dyn Shape]) -> u32 {
    shapes.iter().map(|shape| shape.area()).sum()
}

fn largest<'a, S: Shape + ?Sized>(a: &'a S, b: &'a S) -> &'a S {
    if a.area() >= b.area() { a } else { b }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
struct Vec2 {
    x: i32,
    y: i32,
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Mul<i32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scale: i32) -> Vec2 {
        Vec2 { x: self.x * scale, y: self.y * scale }
    }
}

impl Mul<Vec2> for Vec2 {
    type Output = i32;
    fn mul(self, other: Vec2) -> i32 {
        self.x * other.x + self.y * other.y
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2 { x: -self.x, y: -self.y }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Index<usize> for Vec2 {
    type Output = i32;
    fn index(&self, index: usize) -> &i32 {
        match index {
            0 => &self.x,
            _ => &self.y,
        }
    }
}

impl IndexMut<usize> for Vec2 {
    fn index_mut(&mut self, index: usize) -> &mut i32 {
        match index {
            0 => &mut self.x,
            _ => &mut self.y,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Flags(u8);

impl Not for Flags {
    type Output = Flags;
    fn not(self) -> Flags {
        Flags(!self.0)
    }
}

struct Fnv(u64);

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
    }
}

fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
    let mut hasher = Fnv(0xcbf2_9ce4_8422_2325);
    value.hash(&mut hasher);
    hasher.finish()
}

trait Container {
    type Item;
    const CAPACITY: usize;
    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn first(&self) -> Option<&Self::Item> {
        self.get(0)
    }
}

struct Stack<T, const N: usize> {
    items: [Option<T>; N],
    len: usize,
}

impl<T: Copy, const N: usize> Stack<T, N> {
    fn new() -> Self {
        Stack { items: [None; N], len: 0 }
    }
    fn push(&mut self, item: T) -> Result<(), T> {
        if self.len == N {
            return Err(item);
        }
        self.items[self.len] = Some(item);
        self.len += 1;
        Ok(())
    }
    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.items[self.len].take()
    }
}

impl<T, const N: usize> Container for Stack<T, N> {
    type Item = T;
    const CAPACITY: usize = N;
    fn get(&self, index: usize) -> Option<&T> {
        self.items.get(index).and_then(|slot| slot.as_ref())
    }
}

fn sum_first<C: Container>(container: &C) -> usize
where
    C::Item: Copy + Into<u64>,
{
    container.first().map_or(0, |item| (*item).into() as usize) + C::CAPACITY
}

struct Meters<T>(f64, PhantomData<T>);
struct Metric;

impl<T> Meters<T> {
    fn double(&self) -> Meters<T> {
        Meters(self.0 * 2.0, PhantomData)
    }
}

struct Noisy<'a> {
    id: u8,
    log: &'a Cell<u32>,
}

impl Drop for Noisy<'_> {
    fn drop(&mut self) {
        self.log.set(self.log.get() * 10 + u32::from(self.id));
    }
}

fn apply<F: Fn(i32) -> i32>(f: F, value: i32) -> i32 {
    f(value)
}

fn apply_mut<F: FnMut(i32)>(mut f: F, times: i32) {
    for i in 0..times {
        f(i);
    }
}

fn apply_once<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

fn adder(amount: i32) -> impl Fn(i32) -> i32 {
    move |value| value + amount
}

fn compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C) -> impl Fn(A) -> C {
    move |value| g(f(value))
}

fn twice(f: fn(i32) -> i32, value: i32) -> i32 {
    f(f(value))
}

fn triple(value: i32) -> i32 {
    value * 3
}

fn evens(limit: u32) -> impl Iterator<Item = u32> + Clone {
    (0..limit).filter(|v| v % 2 == 0)
}

trait Animal {
    fn sound(&self) -> &'static str;
}
trait Named: Animal {
    fn title(&self) -> &'static str;
    fn speak(&self, out: &mut dyn Write) -> fmt::Result {
        write!(out, "{} says {}", self.title(), self.sound())
    }
}
struct Dog;
impl Animal for Dog {
    fn sound(&self) -> &'static str {
        "woof"
    }
}
impl Named for Dog {
    fn title(&self) -> &'static str {
        "dog"
    }
}

trait Describe {
    fn describe(&self) -> &'static str;
}
impl<T: Shape> Describe for T {
    fn describe(&self) -> &'static str {
        if self.area() > 20 { "big" } else { "small" }
    }
}

fn run(out: &mut Out) -> fmt::Result {
    let square = Square(4);
    let rect = Rect { w: 3, h: 7 };
    let shapes: [&dyn Shape; 3] = [&square, &rect, &Square(1)];
    for shape in shapes {
        shape.describe(out)?;
        write!(out, " ")?;
    }
    writeln!(out, "total {}", total_area(&shapes))?;
    writeln!(out, "largest {} {}", largest(&square, &Square(5)).area(), largest(shapes[0], shapes[1]).name())?;
    writeln!(out, "blanket {} {}", Describe::describe(&square), Describe::describe(&rect))?;

    let a = Vec2 { x: 3, y: -4 };
    let b = Vec2 { x: 1, y: 2 };
    let mut c = a + b * 2 - -b;
    c += Vec2 { x: 10, y: 10 };
    c[1] *= 2;
    writeln!(out, "ops {:?} {} {} {} {:?}", c, a * b, c[0], c[1], !Flags(0b1010_0101))?;
    writeln!(out, "derive {} {} {:?} {:?} {:?}", a == b, a != a, Vec2::default(), a.max(b), a.cmp(&b))?;
    writeln!(out, "hash {:016x} {:016x} {}", hash_of(&a), hash_of("abc"), hash_of(&a) == hash_of(&Vec2 { x: 3, y: -4 }))?;
    writeln!(out, "hash {:016x} {:016x} {:016x}", hash_of(&(1u8, 'x', true)), hash_of(&[1u16, 2, 3][..]), hash_of(&Some(7u64)))?;

    let mut stack = Stack::<u16, 3>::new();
    writeln!(out, "stack {:?} {:?} {:?} {:?}", stack.push(5), stack.push(6), stack.push(7), stack.push(8))?;
    writeln!(out, "stack {:?} {:?} {} {}", stack.pop(), stack.first(), sum_first(&stack), <Stack<u8, 9> as Container>::CAPACITY)?;
    let distance: Meters<Metric> = Meters(1.25, PhantomData);
    writeln!(out, "phantom {} {}", distance.double().0, size_of::<Meters<Metric>>())?;

    let log = Cell::new(0);
    {
        let _first = Noisy { id: 1, log: &log };
        let second = Noisy { id: 2, log: &log };
        let _third = Noisy { id: 3, log: &log };
        drop(second);
        let moved = Noisy { id: 4, log: &log };
        let _kept = moved;
        let _ = Noisy { id: 5, log: &log };
        core::mem::forget(Noisy { id: 9, log: &log });
    }
    writeln!(out, "drop order {}", log.get())?;

    let offset = 10;
    writeln!(out, "closures {} {} {}", apply(|v| v * v, 7), apply(|v| v + offset, 7), apply(triple, 7))?;
    let mut collected = 0;
    apply_mut(|v| collected += v * v, 5);
    let text = "moved";
    let length = apply_once(move || text.len());
    writeln!(out, "closures {} {} {}", collected, length, apply_once(|| Vec2 { x: 1, y: 1 }).x)?;
    let add5 = adder(5);
    let pipeline = compose(add5, |v| v * 2);
    let stringly = compose(|v: u8| v as char, |c| c.is_ascii_uppercase());
    writeln!(out, "compose {} {} {} {}", pipeline(1), pipeline(-5), stringly(b'Q'), stringly(b'q'))?;
    let table: [(&str, fn(i32) -> i32); 3] = [("triple", triple), ("negate", |v| -v), ("abs", i32::abs)];
    for (name, function) in table {
        write!(out, "{}={} ", name, twice(function, -2))?;
    }
    writeln!(out)?;
    let dynamic: &dyn Fn(i32) -> i32 = &adder(100);
    let mut counter = 0;
    let mut bump = || {
        counter += 1;
        counter
    };
    let dyn_mut: &mut dyn FnMut() -> i32 = &mut bump;
    writeln!(out, "dyn {} {} {}", dynamic(1), dyn_mut(), dyn_mut())?;
    let numbers = evens(10);
    writeln!(out, "impl iter {} {:?}", numbers.clone().count(), numbers.last())?;
    Dog.speak(out)?;
    let named: &dyn Named = &Dog;
    writeln!(out, " / {} {}", named.title(), named.sound())?;
    writeln!(out, "sizes {} {} {}", size_of::<&dyn Shape>(), size_of::<&Square>(), size_of_val(&shapes))?;
    Ok(())
}
