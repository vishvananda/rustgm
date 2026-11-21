fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Seq {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn map<B, F: FnMut(Self::Item) -> B>(self, f: F) -> Mapped<Self, F>
    where
        Self: Sized,
    {
        Mapped { seq: self, f }
    }
}
struct Slice<'a> {
    items: &'a [Predicate],
    at: usize,
}
impl<'a> Seq for Slice<'a> {
    type Item = &'a Predicate;
    fn next(&mut self) -> Option<&'a Predicate> {
        if self.at < self.items.len() {
            self.at += 1;
            Some(&self.items[self.at - 1])
        } else {
            None
        }
    }
}
struct Mapped<S, F> {
    seq: S,
    f: F,
}
impl<B, S: Seq, F: FnMut(S::Item) -> B> Seq for Mapped<S, F> {
    type Item = B;
    fn next(&mut self) -> Option<B> {
        match self.seq.next() {
            Some(item) => Some((self.f)(item)),
            None => None,
        }
    }
}
fn total(mut seq: impl Seq<Item = Input>) -> u8 {
    let mut sum = 0;
    while let Some(input) = seq.next() {
        sum += input.0;
    }
    sum
}
#[derive(Clone, Copy)]
struct Predicate(u8);
#[derive(Clone, Copy)]
struct Input(u8);
trait Conv<T> {
    fn conv(value: T) -> Self;
}
impl Conv<&Predicate> for Input {
    fn conv(predicate: &Predicate) -> Input {
        Input(predicate.0 + 1)
    }
}
fn map_free<S: Seq, B, F: FnMut(S::Item) -> B>(seq: S, f: F) -> Mapped<S, F> {
    Mapped { seq, f }
}
// The implementation's method takes its own late lifetime. A generic
// callee infers `F` from the argument and names that trait method again at
// fresh lifetimes in its result: every copy runs the implementation.
fn by_method<'a>(items: &'a [Predicate]) -> impl Seq<Item = Input> + 'a {
    Slice { items, at: 0 }.map(Input::conv)
}
fn by_path<'a>(items: &'a [Predicate]) -> impl Seq<Item = Input> + 'a {
    Seq::map(Slice { items, at: 0 }, Input::conv)
}
fn by_function<'a>(items: &'a [Predicate]) -> impl Seq<Item = Input> + 'a {
    map_free(Slice { items, at: 0 }, Input::conv)
}
fn direct(items: &[Predicate]) -> u8 {
    total(Slice { items, at: 0 }.map(Input::conv))
}
fn main() {
    let items = [Predicate(1), Predicate(2)];
    check(total(by_method(&items)) == 5);
    check(total(by_path(&items)) == 5);
    check(total(by_function(&items)) == 5);
    check(direct(&items) == 5);
}
