fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}

fn fail(n: u8) -> u8 {
    1 / n
}

trait Source {
    type Item;
    fn one(self) -> Self::Item;
}

trait Value {
    fn make() -> Self;
}

struct First(u8);
struct Second(u8);

impl Value for First {
    fn make() -> Self {
        First(5)
    }
}

impl Value for Second {
    fn make() -> Self {
        Second(7)
    }
}

struct One<T>(T);

impl<T> Source for One<T> {
    type Item = T;
    fn one(self) -> T {
        self.0
    }
}

fn choose<T: Value>() -> One<T> {
    One(T::make())
}

fn inferred() -> impl Source<Item = First> {
    choose::<Second>()
}

fn main() {
    check(inferred().one().0 == 5);
}
