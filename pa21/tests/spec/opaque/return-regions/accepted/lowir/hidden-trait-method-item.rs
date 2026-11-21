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
trait Conv<T>: Sized {
    fn conv(value: T) -> Self;
    fn twice(value: T) -> Self {
        Self::conv(value)
    }
}
impl Conv<&Predicate> for Input {
    fn conv(predicate: &Predicate) -> Input {
        Input(predicate.0 + 1)
    }
    fn twice(predicate: &Predicate) -> Input {
        Input(predicate.0 + 2)
    }
}
// A hidden type names a trait method through its implementing type, as
// `.map(Input::from)` does: the caller executes the implementation's method.
fn inputs<'a>(items: &'a [Predicate]) -> impl Seq<Item = Input> + 'a {
    Mapped { seq: Slice { items, at: 0 }, f: Input::conv }
}
// The same through a parameter: the caller's instance names the method.
fn converted<'a, T: Conv<&'a Predicate> + 'a>(items: &'a [Predicate]) -> impl Seq<Item = T> + 'a {
    Mapped { seq: Slice { items, at: 0 }, f: T::conv }
}
// A defaulted method the implementation overrides runs the override.
fn doubled<'a>(items: &'a [Predicate]) -> impl Seq<Item = Input> + 'a {
    Mapped { seq: Slice { items, at: 0 }, f: Input::twice }
}
fn main() {
    let items = [Predicate(1), Predicate(2)];
    check(total(inputs(&items)) == 5);
    check(total(converted::<Input>(&items)) == 5);
    check(total(doubled(&items)) == 7);
}
