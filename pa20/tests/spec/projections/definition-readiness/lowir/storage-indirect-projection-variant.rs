fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Array {
    type Item;
}
impl<T> Array for [T; 4] {
    type Item = T;
}
struct List<T> {
    first: *const T,
    len: usize,
}
// A variant stores a projection of the parameter behind a pointer only.
enum Small<A: Array> {
    Inline(A, u16),
    Heap(List<A::Item>),
}
struct Decompositions {
    buffer: Small<[(u8, char); 4]>,
    ready: usize,
}
fn main() {
    let pair = (7u8, 'a');
    let heap = Decompositions { buffer: Small::Heap(List { first: &pair, len: 1 }), ready: 0 };
    let inline = Decompositions { buffer: Small::Inline([pair; 4], 3), ready: 1 };
    check(length(heap) == 1 && length(inline) == 4);
}
fn length(d: Decompositions) -> usize {
    match d.buffer {
        Small::Inline(items, n) => {
            check(items[3].1 == 'a');
            n as usize + d.ready
        }
        Small::Heap(list) => {
            check(unsafe { (*list.first).0 } == 7);
            list.len + d.ready
        }
    }
}
