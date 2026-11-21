trait Seq {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

struct Cursor<'a> {
    item: &'a u8,
    done: bool,
}

impl<'a> Seq for Cursor<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            None
        } else {
            self.done = true;
            Some(self.item)
        }
    }
}

fn opaque(item: &u8) -> impl Seq<Item = &u8> + '_ {
    Cursor { item, done: false }
}

fn apply<T, U>(source: &mut T, action: impl FnOnce(&mut T) -> U) -> U {
    action(source)
}

fn advance<T: Seq>(source: &mut T) -> Option<T::Item> {
    apply(source, Seq::next)
}

fn fail(value: u8) -> u8 {
    1 / value
}

fn check(good: bool) {
    if !good {
        let _ = fail(0);
    }
}

fn main() {
    let payload = 17;
    let mut source = opaque(&payload);
    match advance(&mut source) {
        Some(value) => check(*value == 17),
        None => check(false),
    }
    match advance(&mut source) {
        Some(_) => check(false),
        None => {}
    }
}
