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

use alloc::collections::{BTreeMap, BinaryHeap, LinkedList, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Reverse;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Task {
    priority: u8,
    name: &'static str,
}

fn shortest_paths(edges: &[(usize, usize, u32)], nodes: usize, start: usize) -> Vec<Option<u32>> {
    let mut adjacency: Vec<Vec<(usize, u32)>> = alloc::vec![Vec::new(); nodes];
    for &(from, to, cost) in edges {
        adjacency[from].push((to, cost));
        adjacency[to].push((from, cost));
    }
    let mut distance: Vec<Option<u32>> = alloc::vec![None; nodes];
    let mut heap = BinaryHeap::new();
    heap.push(Reverse((0u32, start)));
    while let Some(Reverse((cost, node))) = heap.pop() {
        if distance[node].is_some() {
            continue;
        }
        distance[node] = Some(cost);
        for &(next, weight) in &adjacency[node] {
            if distance[next].is_none() {
                heap.push(Reverse((cost + weight, next)));
            }
        }
    }
    distance
}

fn breadth_first(grid: &[&str], start: (usize, usize), goal: (usize, usize)) -> Option<usize> {
    let rows = grid.len();
    let columns = grid[0].len();
    let mut seen = alloc::vec![alloc::vec![false; columns]; rows];
    let mut queue = VecDeque::new();
    queue.push_back((start, 0));
    seen[start.0][start.1] = true;
    while let Some(((row, column), steps)) = queue.pop_front() {
        if (row, column) == goal {
            return Some(steps);
        }
        let neighbours = [
            (row.wrapping_sub(1), column),
            (row + 1, column),
            (row, column.wrapping_sub(1)),
            (row, column + 1),
        ];
        for (r, c) in neighbours {
            if r < rows && c < columns && !seen[r][c] && grid[r].as_bytes()[c] != b'#' {
                seen[r][c] = true;
                queue.push_back(((r, c), steps + 1));
            }
        }
    }
    None
}

fn run(out: &mut Out) -> fmt::Result {
    let mut deque: VecDeque<i32> = VecDeque::new();
    deque.push_back(1);
    deque.push_back(2);
    deque.push_front(0);
    deque.push_front(-1);
    writeln!(out, "deque {:?} {} {:?} {:?}", deque, deque.len(), deque.front(), deque.back())?;
    writeln!(out, "index {} {:?} {:?} {}", deque[1], deque.get(3), deque.get(4), deque.contains(&2))?;
    writeln!(out, "pop {:?} {:?} {:?}", deque.pop_front(), deque.pop_back(), deque)?;
    let mut ring: VecDeque<u32> = VecDeque::with_capacity(8);
    for value in 0..50 {
        if ring.len() == 6 {
            ring.pop_front();
        }
        ring.push_back(value);
    }
    writeln!(out, "ring {:?} {}", ring, ring.iter().sum::<u32>())?;
    ring.rotate_left(2);
    writeln!(out, "rotate {:?}", ring)?;
    ring.rotate_right(1);
    ring.make_contiguous().sort_unstable_by(|a, b| b.cmp(a));
    writeln!(out, "sorted {:?} {:?}", ring, ring.as_slices().0)?;
    ring.extend([1, 2, 3]);
    ring.insert(1, 100);
    writeln!(out, "insert {:?}", ring)?;
    writeln!(out, "remove {:?} {:?} {:?}", ring.remove(0), ring.swap_remove_back(0), ring)?;
    ring.retain(|v| v % 2 == 1);
    ring.truncate(4);
    writeln!(out, "retain {:?} {:?}", ring, ring.iter().rev().collect::<Vec<_>>())?;
    let drained: Vec<u32> = ring.drain(1..).collect();
    let tail: VecDeque<u32> = drained.iter().map(|v| v * 2).collect();
    ring.append(&mut tail.clone());
    writeln!(out, "drain {:?} {:?} {:?}", drained, ring, ring.range(1..3).collect::<Vec<_>>())?;
    let from_vec: VecDeque<char> = VecDeque::from(alloc::vec!['a', 'b', 'c']);
    let back_to_vec: Vec<char> = from_vec.clone().into();
    writeln!(out, "convert {:?} {:?} {} {:?}", from_vec, back_to_vec, from_vec == ['a', 'b', 'c'], from_vec.iter().collect::<String>())?;
    let mut rotated: VecDeque<u8> = (0..10).collect();
    for step in 1..=7 {
        rotated.rotate_left(step % 10);
        let front = rotated.pop_front().unwrap_or(0);
        rotated.push_back(front);
    }
    writeln!(out, "mix {:?} {:?}", rotated, rotated.iter().position(|v| *v == 0))?;
    writeln!(out, "maze {:?} {:?}", breadth_first(&["..#....", ".##.##.", "....#..", ".##...#", "...#..."], (0, 0), (4, 6)), breadth_first(&[".#.", "###", "..."], (0, 0), (2, 2)))?;

    let mut heap = BinaryHeap::new();
    for value in [5, 1, 8, 3, 9, 2, 8] {
        heap.push(value);
    }
    writeln!(out, "heap {} {:?} {:?}", heap.len(), heap.peek(), heap.clone().into_sorted_vec())?;
    let mut popped = Vec::new();
    while let Some(value) = heap.pop() {
        popped.push(value);
    }
    writeln!(out, "popped {:?} {}", popped, heap.is_empty())?;
    let mut smallest = BinaryHeap::new();
    for value in [15u32, 3, 99, 42, 7, 23, 8, 16, 4] {
        smallest.push(Reverse(value));
    }
    let three: Vec<u32> = (0..3).filter_map(|_| smallest.pop().map(|Reverse(v)| v)).collect();
    writeln!(out, "min-heap {:?} {:?}", three, smallest.peek())?;
    let mut tasks = BinaryHeap::from(alloc::vec![
        Task { priority: 2, name: "write" },
        Task { priority: 9, name: "fix" },
        Task { priority: 2, name: "test" },
    ]);
    tasks.push(Task { priority: 5, name: "review" });
    if let Some(mut top) = tasks.peek_mut() {
        top.priority = 1;
    }
    write!(out, "tasks")?;
    while let Some(task) = tasks.pop() {
        write!(out, " {}:{}", task.priority, task.name)?;
    }
    writeln!(out)?;
    let mut top_k: BinaryHeap<Reverse<u64>> = BinaryHeap::new();
    let mut state = 12345u64;
    for _ in 0..1000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        top_k.push(Reverse(state >> 40));
        if top_k.len() > 5 {
            top_k.pop();
        }
    }
    writeln!(out, "top {:?}", top_k.into_sorted_vec().into_iter().map(|Reverse(v)| v).collect::<Vec<_>>())?;
    let edges = [(0, 1, 7), (0, 2, 9), (0, 5, 14), (1, 2, 10), (1, 3, 15), (2, 3, 11), (2, 5, 2), (3, 4, 6), (4, 5, 9)];
    writeln!(out, "dijkstra {:?}", shortest_paths(&edges, 7, 0))?;

    let mut list: LinkedList<&str> = LinkedList::new();
    list.push_back("b");
    list.push_back("c");
    list.push_front("a");
    writeln!(out, "list {:?} {} {:?} {:?}", list, list.len(), list.front(), list.back())?;
    let mut second = LinkedList::from(["x", "y"]);
    list.append(&mut second);
    let tail = list.split_off(3);
    writeln!(out, "split {:?} {:?} {}", list, tail, second.is_empty())?;
    for item in list.iter_mut() {
        *item = if *item == "b" { "B" } else { item };
    }
    writeln!(out, "pop {:?} {:?} {:?} {}", list.pop_front(), list.pop_back(), list, list.contains(&"B"))?;
    let lengths: BTreeMap<usize, usize> = tail.iter().chain(list.iter()).fold(BTreeMap::new(), |mut acc, item| {
        *acc.entry(item.len()).or_default() += 1;
        acc
    });
    writeln!(out, "fold {:?} {:?}", lengths, tail.into_iter().rev().collect::<Vec<_>>())?;
    Ok(())
}
