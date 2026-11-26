extern crate base;

pub fn denied(n: usize) -> usize {
    base::select(n, 7, true, 4)
}
