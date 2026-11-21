fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Seq {
    type Iter;
    fn start(self) -> Self::Iter;
}
struct Cursor<'a> {
    at: &'a u32,
}
impl<'a> Seq for &'a [u32; 2] {
    type Iter = Cursor<'a>;
    fn start(self) -> Cursor<'a> {
        Cursor { at: &self[1] }
    }
}
struct Noisy(u32);
impl Drop for Noisy {
    fn drop(&mut self) {}
}
trait Peek {
    fn peek(&self) -> u32;
}
// The hidden type stores a projection of its region-carrying parameter, as a
// returned `FlatMap` stores `Option<U::IntoIter>`; a caller stores it at its
// own lifetime.
struct Front<U: Seq> {
    front: Option<U::Iter>,
    noisy: Noisy,
}
impl<'a> Peek for Front<&'a [u32; 2]> {
    fn peek(&self) -> u32 {
        match &self.front {
            Some(c) => *c.at + self.noisy.0,
            None => 0,
        }
    }
}
fn front(a: &[u32; 2]) -> impl Peek + '_ {
    Front::<&[u32; 2]> { front: Some(a.start()), noisy: Noisy(10) }
}
fn main() {
    let a = [1, 2];
    let f = front(&a);
    let g = front(&a);
    check(f.peek() == 12 && g.peek() == 12);
}
