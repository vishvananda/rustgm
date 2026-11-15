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
use alloc::vec::Vec;
use core::any::Any;
use core::cell::RefCell;

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> String;
}

struct Circle(f64);
struct Rect(f64, f64);

impl Shape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.0 * self.0
    }
    fn name(&self) -> String {
        "circle".to_string()
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.0 * self.1
    }
    fn name(&self) -> String {
        alloc::format!("rect {}x{}", self.0, self.1)
    }
}

enum List {
    Cons(i32, Box<List>),
    Nil,
}

impl List {
    fn sum(&self) -> i32 {
        match self {
            List::Cons(value, rest) => value + rest.sum(),
            List::Nil => 0,
        }
    }
    fn from_slice(values: &[i32]) -> List {
        match values {
            [first, rest @ ..] => List::Cons(*first, Box::new(List::from_slice(rest))),
            [] => List::Nil,
        }
    }
}

#[derive(Debug)]
enum Expr {
    Number(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(value) => *value,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
        Expr::Neg(inner) => -eval(inner),
    }
}

fn simplify(expr: Expr) -> Expr {
    match expr {
        Expr::Add(a, b) => match (simplify(*a), simplify(*b)) {
            (Expr::Number(0), other) | (other, Expr::Number(0)) => other,
            (a, b) => Expr::Add(Box::new(a), Box::new(b)),
        },
        Expr::Mul(a, b) => match (simplify(*a), simplify(*b)) {
            (Expr::Number(1), other) | (other, Expr::Number(1)) => other,
            (Expr::Number(0), _) | (_, Expr::Number(0)) => Expr::Number(0),
            (a, b) => Expr::Mul(Box::new(a), Box::new(b)),
        },
        Expr::Neg(inner) => match simplify(*inner) {
            Expr::Neg(twice) => *twice,
            other => Expr::Neg(Box::new(other)),
        },
        number => number,
    }
}

struct Tree {
    value: u32,
    left: Option<Box<Tree>>,
    right: Option<Box<Tree>>,
}

impl Tree {
    fn insert(node: &mut Option<Box<Tree>>, value: u32) {
        match node {
            None => *node = Some(Box::new(Tree { value, left: None, right: None })),
            Some(tree) if value < tree.value => Tree::insert(&mut tree.left, value),
            Some(tree) => Tree::insert(&mut tree.right, value),
        }
    }
    fn walk(node: &Option<Box<Tree>>, visit: &mut dyn FnMut(u32)) {
        if let Some(tree) = node {
            Tree::walk(&tree.left, visit);
            visit(tree.value);
            Tree::walk(&tree.right, visit);
        }
    }
    fn depth(node: &Option<Box<Tree>>) -> usize {
        node.as_ref().map_or(0, |tree| 1 + Tree::depth(&tree.left).max(Tree::depth(&tree.right)))
    }
}

struct Tracked<'a> {
    name: &'static str,
    log: &'a RefCell<Vec<&'static str>>,
}

impl Drop for Tracked<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.name);
    }
}

fn make_counter(step: u32) -> Box<dyn FnMut() -> u32> {
    let mut total = 0;
    Box::new(move || {
        total += step;
        total
    })
}

fn describe(value: &dyn Any) -> String {
    if let Some(number) = value.downcast_ref::<i32>() {
        alloc::format!("i32 {}", number)
    } else if let Some(text) = value.downcast_ref::<String>() {
        alloc::format!("string {:?}", text)
    } else {
        "unknown".to_string()
    }
}

fn run(out: &mut Out) -> fmt::Result {
    let boxed = Box::new(41);
    let mut moved = boxed;
    *moved += 1;
    let unboxed: i32 = *moved;
    writeln!(out, "box {} {} {}", moved, unboxed, size_of::<Box<i32>>() == size_of::<usize>())?;
    let shapes: Vec<Box<dyn Shape>> = alloc::vec![Box::new(Circle(2.0)), Box::new(Rect(3.0, 4.5)), Box::new(Circle(0.5))];
    for shape in &shapes {
        write!(out, "{}={} ", shape.name(), shape.area())?;
    }
    let largest = shapes.iter().max_by(|a, b| a.area().total_cmp(&b.area())).map(|s| s.name());
    writeln!(out, "total {} largest {:?}", shapes.iter().map(|s| s.area()).sum::<f64>(), largest)?;
    let list = List::from_slice(&[1, 2, 3, 4, 5]);
    let mut long = List::Nil;
    for value in 0..2000 {
        long = List::Cons(value, Box::new(long));
    }
    writeln!(out, "list {} {}", list.sum(), long.sum())?;
    let expr = Expr::Add(
        Box::new(Expr::Mul(Box::new(Expr::Number(1)), Box::new(Expr::Number(7)))),
        Box::new(Expr::Neg(Box::new(Expr::Neg(Box::new(Expr::Add(Box::new(Expr::Number(0)), Box::new(Expr::Number(5)))))))),
    );
    let value = eval(&expr);
    let simple = simplify(expr);
    writeln!(out, "expr {} {} {:?}", value, eval(&simple), simple)?;
    let mut root = None;
    for value in [50, 30, 70, 20, 40, 60, 80, 35, 45, 65] {
        Tree::insert(&mut root, value);
    }
    let mut order = Vec::new();
    Tree::walk(&root, &mut |value| order.push(value));
    writeln!(out, "tree {:?} depth {}", order, Tree::depth(&root))?;

    let mut counter = make_counter(5);
    let mut other = make_counter(100);
    writeln!(out, "closures {} {} {} {}", counter(), counter(), other(), counter())?;
    let operations: Vec<(&str, Box<dyn Fn(i32, i32) -> i32>)> = alloc::vec![
        ("add", Box::new(|a, b| a + b)),
        ("mul", Box::new(|a, b| a * b)),
        ("offset", Box::new(move |a, b| a - b + unboxed)),
    ];
    for (name, operation) in &operations {
        write!(out, "{}={} ", name, operation(6, 7))?;
    }
    writeln!(out)?;
    let once: Box<dyn FnOnce() -> String> = Box::new(move || alloc::format!("consumed {}", moved));
    writeln!(out, "{}", once())?;

    let anything: Vec<Box<dyn Any>> = alloc::vec![Box::new(5i32), Box::new(String::from("five")), Box::new(5.0f32)];
    for item in &anything {
        write!(out, "[{}] ", describe(item.as_ref()))?;
    }
    writeln!(out)?;
    let recovered = anything.into_iter().next().and_then(|item| item.downcast::<i32>().ok());
    writeln!(out, "downcast {:?}", recovered)?;

    let log = RefCell::new(Vec::new());
    {
        let _a = Box::new(Tracked { name: "a", log: &log });
        let b = Box::new(Tracked { name: "b", log: &log });
        let items: Vec<Box<Tracked>> = alloc::vec![
            Box::new(Tracked { name: "v0", log: &log }),
            Box::new(Tracked { name: "v1", log: &log }),
        ];
        let pair = (Tracked { name: "t0", log: &log }, Tracked { name: "t1", log: &log });
        drop(b);
        let (first, _second) = pair;
        drop(first);
        let mut slot = Some(Tracked { name: "old", log: &log });
        log.borrow_mut().push(if slot.is_some() { "assign" } else { "empty" });
        slot = Some(Tracked { name: "new", log: &log });
        log.borrow_mut().push("end of scope");
        let _ = (&items, &slot);
    }
    writeln!(out, "drops {:?}", log.borrow())?;

    let slice: Box<[u32]> = (1..=5).collect();
    let text: Box<str> = "boxed str".into();
    let array = Box::new([7u8; 1000]);
    writeln!(out, "unsized {:?} {} {} {}", slice, text, array.len(), array.iter().map(|b| *b as u32).sum::<u32>())?;
    let raw = Box::into_raw(Box::new(String::from("raw")));
    let restored = unsafe {
        (*raw).push_str(" pointer");
        Box::from_raw(raw)
    };
    let leaked: &'static mut u64 = Box::leak(Box::new(99));
    *leaked += 1;
    writeln!(out, "raw {} leak {}", restored, leaked)?;
    let cloned = shapes.iter().map(|s| Box::new(s.area())).collect::<Vec<Box<f64>>>();
    let nested: Box<Box<Box<i8>>> = Box::new(Box::new(Box::new(-3)));
    writeln!(out, "clone {:?} nested {} eq {}", cloned, ***nested, Box::new(1) == Box::new(1))?;
    let uninit = Box::<[u16; 4]>::new_uninit();
    let filled = Box::write(uninit, [1, 2, 3, 4]);
    writeln!(out, "uninit {:?} default {:?}", filled, Box::<Option<u8>>::default())?;
    Ok(())
}
