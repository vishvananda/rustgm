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

use alloc::rc::{Rc, Weak};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;

struct Node {
    name: String,
    parent: RefCell<Weak<Node>>,
    children: RefCell<Vec<Rc<Node>>>,
    log: Rc<RefCell<Vec<String>>>,
}

impl Node {
    fn new(name: &str, log: &Rc<RefCell<Vec<String>>>) -> Rc<Node> {
        Rc::new(Node {
            name: name.to_string(),
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(Vec::new()),
            log: Rc::clone(log),
        })
    }

    fn adopt(parent: &Rc<Node>, child: Rc<Node>) {
        *child.parent.borrow_mut() = Rc::downgrade(parent);
        parent.children.borrow_mut().push(child);
    }

    fn path(&self) -> String {
        match self.parent.borrow().upgrade() {
            Some(parent) => parent.path() + "/" + &self.name,
            None => self.name.clone(),
        }
    }

    fn count(&self) -> usize {
        1 + self.children.borrow().iter().map(|child| child.count()).sum::<usize>()
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        self.log.borrow_mut().push(alloc::format!("drop {}", self.name));
    }
}

struct Account {
    balance: Cell<i64>,
    history: RefCell<Vec<i64>>,
}

fn transfer(from: &Rc<Account>, to: &Rc<Account>, amount: i64) -> bool {
    if from.balance.get() < amount {
        return false;
    }
    from.balance.set(from.balance.get() - amount);
    to.balance.set(to.balance.get() + amount);
    from.history.borrow_mut().push(-amount);
    to.history.borrow_mut().push(amount);
    true
}

struct Owner {
    name: &'static str,
    gadgets: RefCell<Vec<Weak<Gadget>>>,
}

struct Gadget {
    id: u32,
    owner: Rc<Owner>,
}

fn run(out: &mut Out) -> fmt::Result {
    let first = Rc::new(String::from("shared"));
    writeln!(out, "new {} {} {}", first, Rc::strong_count(&first), Rc::weak_count(&first))?;
    let second = Rc::clone(&first);
    let weak = Rc::downgrade(&first);
    writeln!(out, "clone {} {} {} {}", Rc::strong_count(&first), Rc::weak_count(&second), Rc::ptr_eq(&first, &second), first == second)?;
    writeln!(out, "upgrade {:?} {}", weak.upgrade().map(|rc| rc.len()), weak.strong_count())?;
    drop(second);
    writeln!(out, "drop {} {:?}", Rc::strong_count(&first), Rc::try_unwrap(Rc::clone(&first)).map_err(|rc| Rc::strong_count(&rc)))?;
    let unwrapped = Rc::try_unwrap(first);
    writeln!(out, "unwrap {:?} {:?} {}", unwrapped, weak.upgrade(), weak.strong_count())?;

    let mut data = Rc::new(alloc::vec![1, 2, 3]);
    let snapshot = Rc::clone(&data);
    Rc::make_mut(&mut data).push(4);
    writeln!(out, "make_mut {:?} {:?} {}", data, snapshot, Rc::ptr_eq(&data, &snapshot))?;
    let before = Rc::as_ptr(&data);
    Rc::make_mut(&mut data).push(5);
    writeln!(out, "make_mut unique {:?} {}", data, before == Rc::as_ptr(&data))?;
    writeln!(out, "get_mut {:?} {:?}", Rc::get_mut(&mut data).map(|v| v.len()), {
        let alias = Rc::clone(&data);
        let result = Rc::get_mut(&mut data).is_none();
        drop(alias);
        result
    })?;
    writeln!(out, "into_inner {:?} {:?}", Rc::into_inner(Rc::clone(&snapshot)), Rc::into_inner(snapshot))?;

    let text: Rc<str> = Rc::from("unsized str");
    let slice: Rc<[u16]> = Rc::from(alloc::vec![10, 20, 30]);
    let dynamic: Rc<dyn Fn(u32) -> u32> = Rc::new(|v| v * 3);
    let shared_fn = Rc::clone(&dynamic);
    writeln!(out, "unsized {} {} {:?} {} {}", text, text.len(), slice, dynamic(7) + shared_fn(1), Rc::strong_count(&dynamic))?;
    let collected: Rc<[String]> = ["a", "b"].iter().map(|s| s.to_string()).collect();
    writeln!(out, "collected {:?} {}", collected, size_of::<Rc<[String]>>() / size_of::<usize>())?;

    let log = Rc::new(RefCell::new(Vec::new()));
    {
        let root = Node::new("root", &log);
        let usr = Node::new("usr", &log);
        let bin = Node::new("bin", &log);
        let lib = Node::new("lib", &log);
        Node::adopt(&usr, Rc::clone(&bin));
        Node::adopt(&usr, lib);
        Node::adopt(&root, Rc::clone(&usr));
        Node::adopt(&root, Node::new("tmp", &log));
        writeln!(out, "tree {} {} {} {}", bin.path(), root.count(), Rc::strong_count(&usr), Rc::weak_count(&usr))?;
        writeln!(out, "parents {:?} {:?}", bin.parent.borrow().upgrade().map(|p| p.name.clone()), root.parent.borrow().upgrade().is_none())?;
        drop(root);
        writeln!(out, "detached {} {:?}", usr.path(), log.borrow())?;
    }
    writeln!(out, "log {:?} {}", log.borrow(), Rc::strong_count(&log))?;

    let alice = Rc::new(Account { balance: Cell::new(100), history: RefCell::new(Vec::new()) });
    let bob = Rc::new(Account { balance: Cell::new(20), history: RefCell::new(Vec::new()) });
    let ledger = [Rc::clone(&alice), Rc::clone(&bob), Rc::clone(&alice)];
    writeln!(out, "transfer {} {} {}", transfer(&alice, &bob, 70), transfer(&alice, &bob, 70), transfer(&ledger[1], &ledger[2], 5))?;
    writeln!(out, "accounts {} {} {:?} {:?} {}", alice.balance.get(), bob.balance.get(), alice.history.borrow(), bob.history, Rc::strong_count(&alice))?;

    let owner = Rc::new(Owner { name: "ada", gadgets: RefCell::new(Vec::new()) });
    let mut gadgets: Vec<Rc<Gadget>> = (1..=3).map(|id| Rc::new(Gadget { id, owner: Rc::clone(&owner) })).collect();
    for gadget in &gadgets {
        owner.gadgets.borrow_mut().push(Rc::downgrade(gadget));
    }
    gadgets.truncate(1);
    let alive: Vec<u32> = owner.gadgets.borrow().iter().filter_map(Weak::upgrade).map(|g| g.id).collect();
    writeln!(out, "owner {} {:?} {} {}", owner.name, alive, Rc::strong_count(&owner), owner.gadgets.borrow().len())?;
    let survivor = owner.gadgets.borrow()[0].upgrade();
    writeln!(out, "survivor {:?}", survivor.map(|g| (g.id, g.owner.name)))?;

    struct Ring {
        value: u8,
        me: Weak<Ring>,
    }
    let ring = Rc::new_cyclic(|me| Ring { value: 9, me: me.clone() });
    writeln!(out, "cyclic {} {:?} {}", ring.value, ring.me.upgrade().map(|r| r.value), Rc::weak_count(&ring))?;
    let dangling: Weak<u8> = Weak::new();
    writeln!(out, "dangling {:?} {}", dangling.upgrade(), dangling.strong_count())?;

    let leak_a = Rc::new(RefCell::new(None::<Rc<RefCell<Option<Rc<()>>>>>));
    let cells: Vec<Rc<Cell<u32>>> = (0..4).map(|v| Rc::new(Cell::new(v))).collect();
    let aliases = cells.clone();
    for cell in &aliases {
        cell.set(cell.get() * 10 + 1);
    }
    writeln!(out, "aliases {:?} {} {}", cells.iter().map(|c| c.get()).collect::<Vec<_>>(), Rc::strong_count(&cells[0]), leak_a.borrow().is_none())?;
    let raw = Rc::into_raw(Rc::new(77u64));
    let restored = unsafe { Rc::from_raw(raw) };
    writeln!(out, "raw {} {:?} {:?}", restored, Rc::new(5).cmp(&Rc::new(6)), Rc::new("x").to_string())?;
    Ok(())
}
