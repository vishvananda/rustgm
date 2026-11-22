struct Value<T>(T);

trait Stream: Sized {
    type Item;

    fn item(self) -> Self::Item;

    fn follow<S: Stream<Item = Self::Item>>(self, other: S) -> Self::Item {
        other.item()
    }
}

impl<T> Stream for Value<T> {
    type Item = T;

    fn item(self) -> Self::Item {
        self.0
    }
}

fn observed(left: Value<&u8>, right: Value<&u8>) -> u8 {
    *left.follow(right)
}

fn fail(zero: u8) -> u8 {
    1 / zero
}

fn main() {
    let left = 7;
    let right = 9;
    if observed(Value(&left), Value(&right)) != 9 {
        let _ = fail(0);
    }
}
