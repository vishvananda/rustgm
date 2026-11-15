// Atomic integers, booleans and pointers of every width core offers on x86-64.
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

use core::sync::atomic::{
    AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicPtr, AtomicU8, AtomicU16, AtomicU32,
    AtomicU64, AtomicUsize, Ordering,
};

static COUNTER: AtomicUsize = AtomicUsize::new(0);
static FLAG: AtomicBool = AtomicBool::new(false);
static TOTAL: AtomicI64 = AtomicI64::new(-5);
static SMALL: AtomicU8 = AtomicU8::new(200);
static WORD: AtomicU32 = AtomicU32::new(0xffff_fff0);
static SLOT: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());

struct SpinLock {
    locked: AtomicBool,
    acquisitions: AtomicU16,
}

impl SpinLock {
    fn with<R>(&self, f: impl FnOnce() -> R) -> R {
        while self.locked.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            core::hint::spin_loop();
        }
        self.acquisitions.fetch_add(1, Ordering::Relaxed);
        let result = f();
        self.locked.store(false, Ordering::Release);
        result
    }
}

static LOCK: SpinLock = SpinLock { locked: AtomicBool::new(false), acquisitions: AtomicU16::new(0) };
static WIDE: AtomicU64 = AtomicU64::new(u64::MAX - 1);
static SIGNED: [AtomicI8; 2] = [AtomicI8::new(-128), AtomicI8::new(127)];
static SIZE: AtomicIsize = AtomicIsize::new(-1);
static NARROW: AtomicI32 = AtomicI32::new(i32::MIN);
static SHORT: AtomicI16 = AtomicI16::new(-2);

fn next_id() -> usize {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn run(out: &mut Out) -> fmt::Result {
    writeln!(out, "ids {} {} {} {}", next_id(), next_id(), next_id(), COUNTER.load(Ordering::SeqCst))?;
    writeln!(out, "flag {} {} {}", FLAG.swap(true, Ordering::AcqRel), FLAG.load(Ordering::Acquire), FLAG.fetch_xor(true, Ordering::SeqCst))?;
    writeln!(out, "total {} {} {}", TOTAL.fetch_sub(10, Ordering::SeqCst), TOTAL.fetch_max(3, Ordering::Relaxed), TOTAL.load(Ordering::Relaxed))?;
    writeln!(
        out,
        "exchange {:?} {:?} {}",
        TOTAL.compare_exchange(3, 40, Ordering::SeqCst, Ordering::Relaxed),
        TOTAL.compare_exchange(3, 50, Ordering::AcqRel, Ordering::Acquire),
        TOTAL.load(Ordering::SeqCst)
    )?;
    writeln!(out, "small {} {} {}", SMALL.fetch_add(100, Ordering::Relaxed), SMALL.fetch_and(0x0f, Ordering::Relaxed), SMALL.fetch_or(0x80, Ordering::Relaxed))?;
    writeln!(out, "small {} {}", SMALL.fetch_nand(0xf0, Ordering::Relaxed), SMALL.load(Ordering::Relaxed))?;
    writeln!(out, "word {:#x} {:#x}", WORD.fetch_add(0x20, Ordering::Release), WORD.load(Ordering::Acquire))?;
    let mut spins = 0;
    let updated = WORD.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
        spins += 1;
        value.checked_mul(3)
    });
    writeln!(out, "update {:?} {} {}", updated, spins, WORD.load(Ordering::SeqCst))?;
    loop {
        let current = COUNTER.load(Ordering::Relaxed);
        if COUNTER.compare_exchange_weak(current, current * 7, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
            break;
        }
    }
    writeln!(out, "weak {}", COUNTER.load(Ordering::Relaxed))?;
    let mut cell = 77u32;
    writeln!(out, "ptr {}", SLOT.load(Ordering::Acquire).is_null())?;
    SLOT.store(&mut cell, Ordering::Release);
    let stored = SLOT.swap(core::ptr::null_mut(), Ordering::AcqRel);
    writeln!(out, "ptr {} {}", unsafe { *stored }, SLOT.load(Ordering::Acquire).is_null())?;
    FLAG.store(false, Ordering::Release);
    core::sync::atomic::fence(Ordering::SeqCst);
    core::hint::spin_loop();
    writeln!(out, "into_inner {} {}", AtomicU32::new(5).into_inner(), *AtomicBool::new(true).get_mut())?;
    let inside = LOCK.with(|| LOCK.locked.load(Ordering::Relaxed));
    let nested = LOCK.with(|| COUNTER.fetch_add(1, Ordering::SeqCst));
    writeln!(out, "lock {} {} {} {}", inside, nested, LOCK.locked.load(Ordering::SeqCst), LOCK.acquisitions.load(Ordering::Relaxed))?;
    writeln!(out, "wide {} {} {}", WIDE.fetch_add(3, Ordering::SeqCst), WIDE.load(Ordering::SeqCst), WIDE.swap(1 << 40, Ordering::SeqCst))?;
    writeln!(out, "wide {:?} {}", WIDE.compare_exchange(1 << 40, 9, Ordering::Release, Ordering::Relaxed), WIDE.fetch_min(4, Ordering::AcqRel))?;
    writeln!(out, "signed {} {} {}", SIGNED[0].fetch_sub(1, Ordering::Relaxed), SIGNED[1].fetch_add(1, Ordering::Relaxed), SIGNED[0].load(Ordering::Relaxed))?;
    writeln!(out, "signed {} {}", SIGNED[1].fetch_max(0, Ordering::Relaxed), SIGNED[1].fetch_min(-100, Ordering::Relaxed))?;
    writeln!(out, "size {} {} {}", SIZE.fetch_add(isize::MAX, Ordering::Relaxed), SIZE.fetch_and(0xff, Ordering::Relaxed), SIZE.load(Ordering::Relaxed))?;
    writeln!(out, "narrow {} {} {}", NARROW.fetch_sub(1, Ordering::AcqRel), NARROW.fetch_xor(-1, Ordering::AcqRel), NARROW.load(Ordering::Acquire))?;
    writeln!(out, "short {} {} {}", SHORT.fetch_or(1, Ordering::SeqCst), SHORT.swap(300, Ordering::SeqCst), SHORT.load(Ordering::SeqCst))?;
    let local = AtomicUsize::new(10);
    let borrowed = &local;
    let add = |amount: usize| borrowed.fetch_add(amount, Ordering::Relaxed);
    writeln!(out, "local {} {} {} {:?}", add(1), add(2), local.load(Ordering::Relaxed), local)?;
    let mut owned = AtomicU64::new(7);
    *owned.get_mut() *= 6;
    writeln!(out, "owned {} {:?}", owned.load(Ordering::Relaxed), AtomicBool::new(true))?;
    writeln!(out, "sizes {} {} {} {}", size_of::<AtomicU8>(), size_of::<AtomicU16>(), size_of::<AtomicU64>(), align_of::<AtomicU64>())?;
    Ok(())
}
