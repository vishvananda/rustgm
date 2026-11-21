// A generic caller's instance opens its own lifetime for the opaque's: the
// hidden type Wrap<<S as Seq<'a>>::Cursor> is normalized at that lifetime.
fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Next {
    fn next(&mut self) -> Option<u32>;
}
trait Seq<'a> {
    type Cursor: Next;
    fn cursor(self) -> Self::Cursor;
}
struct Cursor<'a> {
    items: &'a [u32; 2],
    at: usize,
}
impl<'a> Next for Cursor<'a> {
    fn next(&mut self) -> Option<u32> {
        if self.at < 2 {
            self.at += 1;
            Some(self.items[self.at - 1])
        } else {
            None
        }
    }
}
impl<'a> Seq<'a> for &'a [u32; 2] {
    type Cursor = Cursor<'a>;
    fn cursor(self) -> Cursor<'a> {
        Cursor { items: self, at: 0 }
    }
}
struct Wrap<C>(C);
impl<C: Next> Next for Wrap<C> {
    fn next(&mut self) -> Option<u32> {
        self.0.next()
    }
}
// The hidden type `Wrap<<S as Seq<'a>>::Cursor>` stores a projection.
fn wrapped<'a, S: Seq<'a>>(s: S) -> impl Next {
    Wrap(s.cursor())
}
fn total<'a, S: Seq<'a>>(s: S) -> u32 {
    let mut w = wrapped(s);
    let mut t = 0;
    while let Some(x) = w.next() {
        t += x;
    }
    t
}
fn main() {
    let a = [3u32, 4];
    check(total(&a) == 7);
}
