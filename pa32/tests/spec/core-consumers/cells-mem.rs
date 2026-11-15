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

use core::cell::{Cell, LazyCell, OnceCell, RefCell, UnsafeCell};
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr::{self, NonNull};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct Header {
    tag: u8,
    length: u32,
    flags: u16,
}

#[repr(C)]
union Bits {
    float: f32,
    word: u32,
    bytes: [u8; 4],
}

#[derive(Debug)]
#[allow(dead_code)]
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(&'static str),
}

struct Node<'a> {
    value: Cell<i32>,
    next: Cell<Option<&'a Node<'a>>>,
}

fn sum_list(mut node: Option<&Node<'_>>) -> i32 {
    let mut total = 0;
    while let Some(current) = node {
        total += current.value.get();
        node = current.next.get();
    }
    total
}

struct Counter {
    hits: UnsafeCell<u32>,
}

impl Counter {
    fn hit(&self) -> u32 {
        unsafe {
            *self.hits.get() += 1;
            *self.hits.get()
        }
    }
}

fn run(out: &mut Out) -> fmt::Result {
    let cell = Cell::new(5);
    cell.set(cell.get() * 2);
    let old = cell.replace(99);
    writeln!(out, "cell {} {} {} {}", old, cell.get(), cell.take(), cell.into_inner())?;
    let shared = RefCell::new([1, 2, 3]);
    {
        let first = shared.borrow();
        let second = shared.borrow();
        writeln!(out, "refcell {} {} {}", first[0] + second[2], shared.try_borrow_mut().is_err(), shared.try_borrow().is_ok())?;
    }
    shared.borrow_mut()[1] = 20;
    {
        let mut exclusive = shared.borrow_mut();
        exclusive[0] += 100;
        writeln!(out, "refcell {} {}", shared.try_borrow().is_err(), shared.try_borrow_mut().is_err())?;
    }
    writeln!(out, "refcell {:?} {:?}", shared, shared.replace([7, 8, 9]))?;
    writeln!(out, "refcell {:?}", shared.into_inner())?;
    let once = OnceCell::new();
    writeln!(out, "once {:?} {:?} {:?} {}", once.get(), once.set(3), once.set(4), once.get_or_init(|| 5))?;
    let evaluations = Cell::new(0);
    let lazy = LazyCell::new(|| {
        evaluations.set(evaluations.get() + 1);
        6 * 7
    });
    writeln!(out, "lazy {} {} {} {}", evaluations.get(), *lazy, *lazy + 1, evaluations.get())?;
    let third = Node { value: Cell::new(3), next: Cell::new(None) };
    let second = Node { value: Cell::new(2), next: Cell::new(Some(&third)) };
    let first = Node { value: Cell::new(1), next: Cell::new(Some(&second)) };
    third.value.set(30);
    writeln!(out, "list {}", sum_list(Some(&first)))?;
    second.next.set(None);
    writeln!(out, "list {}", sum_list(Some(&first)))?;
    let counter = Counter { hits: UnsafeCell::new(0) };
    writeln!(out, "unsafe cell {} {} {}", counter.hit(), counter.hit(), counter.hits.into_inner())?;

    let mut a = [1, 2, 3];
    let mut b = [9, 8, 7];
    mem::swap(&mut a, &mut b);
    let previous = mem::replace(&mut a[0], 50);
    let taken = mem::take(&mut b[2]);
    writeln!(out, "mem {:?} {:?} {} {}", a, b, previous, taken)?;
    writeln!(
        out,
        "sizes {} {} {} {} {} {}",
        size_of::<Header>(),
        align_of::<Header>(),
        size_of::<Bits>(),
        size_of::<Option<&u8>>(),
        size_of::<Option<u32>>(),
        size_of::<[u16; 5]>()
    )?;
    writeln!(
        out,
        "sizes {} {} {} {} {}",
        size_of::<Message>(),
        size_of::<(u8, u32)>(),
        size_of::<&str>(),
        size_of_val("hello"),
        align_of_val(&0u64)
    )?;
    writeln!(
        out,
        "offsets {} {} {}",
        mem::offset_of!(Header, tag),
        mem::offset_of!(Header, length),
        mem::offset_of!(Header, flags)
    )?;
    let messages = [Message::Quit, Message::Move { x: 1, y: -1 }, Message::Write("hi"), Message::Quit];
    writeln!(
        out,
        "discriminant {} {} {:?}",
        mem::discriminant(&messages[0]) == mem::discriminant(&messages[3]),
        mem::discriminant(&messages[0]) == mem::discriminant(&messages[1]),
        messages
    )?;
    writeln!(out, "needs_drop {} {}", mem::needs_drop::<Header>(), mem::needs_drop::<ManuallyDrop<Header>>())?;

    let bits = Bits { float: 1.0 };
    writeln!(out, "union {:#x} {:?}", unsafe { bits.word }, unsafe { bits.bytes })?;
    let transmuted: u32 = unsafe { mem::transmute(Bits { float: -2.5 }) };
    let array: [u16; 2] = unsafe { mem::transmute([1u8, 2, 3, 4]) };
    writeln!(out, "transmute {:#x} {:?} {}", transmuted, array, f32::from_bits(transmuted))?;

    let mut uninit = MaybeUninit::<Header>::uninit();
    let header = unsafe {
        uninit.as_mut_ptr().write(Header { tag: 7, length: 1000, flags: 3 });
        uninit.assume_init()
    };
    let mut table: [MaybeUninit<u16>; 6] = [MaybeUninit::uninit(); 6];
    for (index, slot) in table.iter_mut().enumerate() {
        slot.write(index as u16 * 11);
    }
    let table: [u16; 6] = unsafe { mem::transmute(table) };
    writeln!(out, "uninit {:?} {:?}", header, table)?;
    let zeroed: Header = unsafe { mem::zeroed() };
    writeln!(out, "zeroed {:?}", zeroed)?;

    let mut values = [10u32, 20, 30, 40, 50];
    let base = values.as_mut_ptr();
    unsafe {
        *base.add(1) += 5;
        ptr::swap(base, base.add(4));
        ptr::copy(base.add(1), base.add(2), 2);
        let read = ptr::read(base.add(3));
        ptr::write(base.add(3), read * 2);
        writeln!(out, "pointers {:?} {} {}", values, base.add(4).offset_from(base), *base.add(4).sub(2))?;
    }
    let mut other = [0u32; 5];
    unsafe {
        ptr::copy_nonoverlapping(values.as_ptr(), other.as_mut_ptr(), 3);
        ptr::write_bytes(other.as_mut_ptr().add(3), 0xff, 1);
    }
    writeln!(out, "copies {:?}", other)?;
    let slice = unsafe { core::slice::from_raw_parts(values.as_ptr().add(1), 3) };
    writeln!(out, "raw slice {:?} {}", slice, ptr::eq(slice.as_ptr(), &values[1]))?;
    let mut target = 5i64;
    let mut non_null = NonNull::from(&mut target);
    unsafe {
        *non_null.as_mut() += 1;
        *non_null.as_ptr() *= 3;
    }
    writeln!(out, "non-null {} {} {}", target, NonNull::<u64>::dangling().as_ptr().addr(), NonNull::new(ptr::null_mut::<u8>()).is_none())?;
    let field = &raw const header.length;
    let unaligned = unsafe { ptr::read_unaligned(field.cast::<u8>().cast::<u32>()) };
    writeln!(out, "raw field {} {}", unsafe { *field }, unaligned)?;
    writeln!(out, "addresses {} {}", ptr::addr_eq(&values[0], values.as_ptr()), (&raw const values[2]).addr() - (&raw const values[0]).addr())?;
    let wide: &[u32] = &values;
    let (data, len) = (wide.as_ptr(), wide.len());
    let rebuilt = ptr::slice_from_raw_parts(data, len);
    writeln!(out, "wide {} {}", rebuilt.len(), unsafe { (&*rebuilt)[4] })?;
    let manual = ManuallyDrop::new([1u8, 2]);
    writeln!(out, "manual {:?} {}", *manual, mem::align_of::<u128>() >= 8)?;
    let erased: *const () = (&raw const target).cast();
    writeln!(out, "erased {}", unsafe { *erased.cast::<i64>() })?;
    Ok(())
}
