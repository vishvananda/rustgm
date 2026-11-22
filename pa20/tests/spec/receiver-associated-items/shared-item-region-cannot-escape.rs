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

fn invalid(left: Value<&'static u8>, right: Value<&u8>) -> &'static u8 {
    left.follow(right)
}

fn main() {}
