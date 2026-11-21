// A caller stores an opaque value whose hidden type, Wrap<<&'a [u32; 2] as
// Seq<'a>>::Cursor>, names a projection at the caller's own lifetime.
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
fn main() {
    let a = [3u32, 4];
    let mut w = wrapped(&a);
    let mut t = 0;
    while let Some(x) = w.next() {
        t += x;
    }
    check(t == 7);
}
