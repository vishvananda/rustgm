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

use alloc::string::{String, ToString};
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, input: u64) -> u64;
}

struct Doubler;
struct Offset {
    amount: u64,
    calls: AtomicUsize,
}

impl Plugin for Doubler {
    fn name(&self) -> &str {
        "doubler"
    }
    fn run(&self, input: u64) -> u64 {
        input * 2
    }
}

impl Plugin for Offset {
    fn name(&self) -> &str {
        "offset"
    }
    fn run(&self, input: u64) -> u64 {
        self.calls.fetch_add(1, Ordering::Relaxed);
        input + self.amount
    }
}

struct Registry {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl Registry {
    fn pipeline(&self, input: u64) -> u64 {
        self.plugins.iter().fold(input, |value, plugin| plugin.run(value))
    }
}

struct Guarded {
    dropped: Arc<AtomicBool>,
    value: u32,
}

impl Drop for Guarded {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

struct Interner {
    names: Vec<Arc<str>>,
}

impl Interner {
    fn intern(&mut self, name: &str) -> Arc<str> {
        if let Some(existing) = self.names.iter().find(|candidate| &***candidate == name) {
            return Arc::clone(existing);
        }
        let created: Arc<str> = Arc::from(name);
        self.names.push(Arc::clone(&created));
        created
    }
}

fn run(out: &mut Out) -> fmt::Result {
    let first = Arc::new(String::from("shared"));
    let second = Arc::clone(&first);
    let weak = Arc::downgrade(&first);
    writeln!(out, "counts {} {} {} {}", Arc::strong_count(&first), Arc::weak_count(&first), Arc::ptr_eq(&first, &second), first == second)?;
    writeln!(out, "upgrade {:?} {} {}", weak.upgrade().map(|arc| arc.len()), weak.strong_count(), weak.weak_count())?;
    drop(second);
    writeln!(out, "try_unwrap {:?}", Arc::try_unwrap(Arc::clone(&first)).map_err(|arc| Arc::strong_count(&arc)))?;
    let inner = Arc::try_unwrap(first);
    writeln!(out, "unwrapped {:?} {:?} {}", inner, weak.upgrade(), weak.strong_count())?;

    let mut data = Arc::new(alloc::vec![1u8, 2, 3]);
    let snapshot = Arc::clone(&data);
    Arc::make_mut(&mut data).push(4);
    writeln!(out, "make_mut {:?} {:?} {}", data, snapshot, Arc::ptr_eq(&data, &snapshot))?;
    writeln!(out, "get_mut {} {}", Arc::get_mut(&mut data).is_some(), {
        let alias = Arc::clone(&data);
        let unique = Arc::get_mut(&mut data).is_some();
        drop(alias);
        unique
    })?;
    writeln!(out, "into_inner {:?} {:?}", Arc::into_inner(Arc::clone(&snapshot)), Arc::into_inner(snapshot))?;

    let counter = Arc::new(AtomicUsize::new(0));
    let handles: Vec<Arc<AtomicUsize>> = (0..8).map(|_| Arc::clone(&counter)).collect();
    for (index, handle) in handles.iter().enumerate() {
        handle.fetch_add(index + 1, Ordering::SeqCst);
    }
    writeln!(out, "atomic {} {}", counter.load(Ordering::SeqCst), Arc::strong_count(&counter))?;
    drop(handles);
    writeln!(out, "after drop {}", Arc::strong_count(&counter))?;

    let offset = Arc::new(Offset { amount: 5, calls: AtomicUsize::new(0) });
    let registry = Registry { plugins: alloc::vec![Arc::new(Doubler), offset.clone(), Arc::new(Doubler), offset.clone()] };
    let names: Vec<&str> = registry.plugins.iter().map(|plugin| plugin.name()).collect();
    writeln!(out, "plugins {:?} {} {}", names, registry.pipeline(10), registry.pipeline(0))?;
    writeln!(out, "calls {} {}", offset.calls.load(Ordering::Relaxed), Arc::strong_count(&offset))?;
    drop(registry);
    writeln!(out, "released {}", Arc::strong_count(&offset))?;

    let dropped = Arc::new(AtomicBool::new(false));
    let guarded = Arc::new(Guarded { dropped: Arc::clone(&dropped), value: 3 });
    let observer = Arc::downgrade(&guarded);
    let copies = [Arc::clone(&guarded), Arc::clone(&guarded)];
    drop(guarded);
    writeln!(out, "guard {} {:?}", dropped.load(Ordering::SeqCst), observer.upgrade().map(|g| g.value))?;
    drop(copies);
    writeln!(out, "guard {} {:?} {}", dropped.load(Ordering::SeqCst), observer.upgrade().map(|g| g.value), observer.strong_count())?;

    let mut interner = Interner { names: Vec::new() };
    let a = interner.intern("alpha");
    let b = interner.intern("beta");
    let again = interner.intern("alpha");
    writeln!(out, "intern {} {} {} {} {}", a, b, Arc::ptr_eq(&a, &again), Arc::ptr_eq(&a, &b), Arc::strong_count(&a))?;
    let slice: Arc<[u32]> = Arc::from(alloc::vec![1, 2, 3]);
    let from_iter: Arc<[String]> = (0..3).map(|v| v.to_string()).collect();
    let boxed: Arc<str> = Arc::from(String::from("from string"));
    writeln!(out, "unsized {:?} {:?} {} {}", slice, from_iter, boxed, slice.iter().sum::<u32>())?;
    let callable: Arc<dyn Fn(&str) -> usize + Send + Sync> = Arc::new(|text| text.len());
    writeln!(out, "callable {} {}", callable("four"), Arc::clone(&callable)(""))?;

    struct Link {
        id: u8,
        me: Weak<Link>,
    }
    let link = Arc::new_cyclic(|me| Link { id: 4, me: me.clone() });
    writeln!(out, "cyclic {} {:?} {}", link.id, link.me.upgrade().map(|l| l.id), Arc::weak_count(&link))?;
    let raw = Arc::into_raw(Arc::new(123u64));
    let restored = unsafe {
        Arc::increment_strong_count(raw);
        let extra = Arc::from_raw(raw);
        let count = Arc::strong_count(&extra);
        drop(extra);
        (Arc::from_raw(raw), count)
    };
    writeln!(out, "raw {} {} {}", restored.0, restored.1, Arc::strong_count(&restored.0))?;
    let ordered = [Arc::new(3), Arc::new(1), Arc::new(2)];
    let mut sorted = ordered.clone();
    sorted.sort();
    writeln!(out, "ordered {:?} {:?} {}", sorted, Arc::new(1).max(Arc::new(2)), size_of::<Arc<u8>>() == size_of::<usize>())?;
    let dangling: Weak<String> = Weak::new();
    writeln!(out, "dangling {:?} {}", dangling.upgrade(), dangling.strong_count())?;
    Ok(())
}
