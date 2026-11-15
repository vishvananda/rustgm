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

use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc};
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::hint::black_box;

#[repr(align(64))]
struct CacheLine([u8; 64]);

#[repr(align(4096))]
struct Page([u8; 4096]);

struct Live {
    blocks: usize,
    bytes: usize,
}

fn live() -> Live {
    Live { blocks: HEAP.blocks.get(), bytes: HEAP.bytes.get() }
}

fn delta(before: &Live) -> (isize, isize) {
    let now = live();
    (now.blocks as isize - before.blocks as isize, now.bytes as isize - before.bytes as isize)
}

fn reserve(amount: usize) -> Result<usize, TryReserveError> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.try_reserve_exact(amount)?;
    Ok(bytes.capacity())
}

fn run(out: &mut Out) -> fmt::Result {
    let start = live();
    writeln!(out, "start {} {}", start.blocks, start.bytes)?;
    let boxed = black_box(Box::new(0x1122_3344_5566_7788u64));
    writeln!(out, "box u64 {:?} {}", delta(&start), (&raw const *boxed).addr() % 8)?;
    let array = black_box(Box::new([0u32; 25]));
    writeln!(out, "box array {:?}", delta(&start))?;
    drop(boxed);
    drop(array);
    writeln!(out, "dropped {:?}", delta(&start))?;

    let vector: Vec<u16> = black_box(Vec::with_capacity(10));
    writeln!(out, "with_capacity {:?} {}", delta(&start), vector.capacity())?;
    let mut vector = vector;
    vector.extend(0..10);
    writeln!(out, "filled {:?}", delta(&start))?;
    vector.push(10);
    let grown = delta(&start);
    writeln!(out, "grown {} {} {}", grown.0, grown.1 as usize == vector.capacity() * 2, vector.capacity() >= 11)?;
    vector.shrink_to_fit();
    writeln!(out, "shrunk {:?} {}", delta(&start), vector.capacity())?;
    let boxed_slice = vector.into_boxed_slice();
    writeln!(out, "boxed slice {:?} {}", delta(&start), boxed_slice.len())?;
    drop(boxed_slice);

    let text = black_box(String::from("twelve bytes"));
    let copy = black_box(text.clone());
    writeln!(out, "strings {:?} {} {}", delta(&start), text.capacity(), copy.as_ptr() != text.as_ptr())?;
    drop((text, copy));
    let empty_vec: Vec<u64> = black_box(Vec::new());
    let empty_string = black_box(String::new());
    let unit_box = black_box(Box::new(()));
    let unit_vec = black_box(alloc::vec![(); 100]);
    writeln!(out, "no allocation {:?} {} {} {}", delta(&start), empty_vec.capacity(), empty_string.capacity(), unit_vec.len())?;
    drop((empty_vec, empty_string, unit_box, unit_vec));

    let line = black_box(Box::new(CacheLine([7; 64])));
    let page = black_box(Box::new(Page([1; 4096])));
    writeln!(
        out,
        "aligned {:?} {} {} {} {}",
        delta(&start),
        (&raw const *line).addr() % 64,
        (&raw const *page).addr() % 4096,
        line.0.iter().map(|b| *b as u32).sum::<u32>(),
        page.0[4095]
    )?;
    let lines: Vec<CacheLine> = (0..5u8).map(|v| CacheLine([v; 64])).collect();
    writeln!(out, "aligned vec {} {} {}", lines.as_ptr().addr() % 64, size_of::<CacheLine>(), lines[4].0[63])?;
    drop((line, page, lines));
    writeln!(out, "released {:?}", delta(&start))?;

    let layout = Layout::new::<u64>();
    let array_layout = Layout::array::<u32>(12).map_err(|_| fmt::Error)?;
    let custom = Layout::from_size_align(24, 8).map_err(|_| fmt::Error)?;
    writeln!(out, "layout {} {} {} {} {:?}", layout.size(), layout.align(), array_layout.size(), array_layout.align(), custom)?;
    writeln!(
        out,
        "layout errors {} {} {}",
        Layout::from_size_align(8, 3).is_err(),
        Layout::from_size_align(usize::MAX, 8).is_err(),
        Layout::array::<u64>(usize::MAX / 4).is_err()
    )?;
    let (extended, offset) = Layout::new::<u8>().extend(Layout::new::<u32>()).map_err(|_| fmt::Error)?;
    writeln!(out, "extend {:?} {} {:?}", extended, offset, extended.pad_to_align())?;
    writeln!(out, "for_value {:?} {:?}", Layout::for_value(&[0u16; 7]), Layout::for_value("hello"))?;
    unsafe {
        let pointer = alloc(custom);
        pointer.write_bytes(0xab, 24);
        let grown = realloc(pointer, custom, 4000);
        let kept = (0..24).all(|index| *grown.add(index) == 0xab);
        let larger = Layout::from_size_align(4000, 8).map_err(|_| fmt::Error)?;
        writeln!(out, "manual {:?} {} {}", delta(&start), kept, grown.addr() % 8)?;
        let shrunk = realloc(grown, larger, 8);
        writeln!(out, "manual {:?} {}", delta(&start), *shrunk)?;
        dealloc(shrunk, Layout::from_size_align(8, 8).map_err(|_| fmt::Error)?);
        let zeroed = alloc_zeroed(array_layout);
        let all_zero = (0..48).all(|index| *zeroed.add(index) == 0);
        dealloc(zeroed, array_layout);
        writeln!(out, "zeroed {} {:?}", all_zero, delta(&start))?;
    }

    writeln!(out, "reserve {:?}", reserve(1000))?;
    writeln!(out, "overflow {:?}", reserve(usize::MAX).map_err(|error| error.to_string()))?;
    writeln!(out, "refused {:?}", reserve(isize::MAX as usize - 4096).map_err(|error| error.to_string()))?;
    let mut strings: Vec<String> = Vec::new();
    writeln!(out, "try_reserve {:?} {}", strings.try_reserve(usize::MAX / 8).is_err(), strings.try_reserve(4).is_ok())?;
    let mut text = String::new();
    writeln!(out, "string {:?} {:?}", text.try_reserve(usize::MAX).is_err(), text.try_reserve(16).map(|()| text.capacity() >= 16))?;
    drop((strings, text));

    let mut churn: Vec<Vec<u8>> = Vec::new();
    let mut state = 99u32;
    let mut peak = 0;
    for round in 0..2000 {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        let size = (state >> 16) as usize % 300;
        if round % 3 == 2 && !churn.is_empty() {
            let index = size % churn.len();
            churn.swap_remove(index);
        } else {
            churn.push(alloc::vec![round as u8; size]);
        }
        peak = peak.max(HEAP.blocks.get());
    }
    let checksum: u64 = churn.iter().map(|block| block.iter().map(|b| u64::from(*b)).sum::<u64>() + block.len() as u64).sum();
    writeln!(out, "churn {} {} {}", churn.len(), checksum, peak > churn.len())?;
    drop(churn);
    writeln!(out, "end {:?}", delta(&start))?;
    let leaked = Box::leak(Box::new([5u8; 100]));
    core::mem::forget(alloc::vec![1u32, 2, 3]);
    writeln!(out, "leaked {} {:?}", leaked.len(), delta(&start))?;
    Ok(())
}
