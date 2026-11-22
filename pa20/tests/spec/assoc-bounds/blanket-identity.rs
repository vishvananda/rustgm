trait Convert<T> {
    fn convert(value: T) -> Self;
}

impl<T> Convert<T> for T {
    fn convert(value: T) -> Self {
        value
    }
}

struct Count(u32);

impl Convert<u32> for Count {
    fn convert(value: u32) -> Self {
        Self(value + 1)
    }
}

trait Family {
    type Value: Convert<u32>;
}

fn identity<F: Family>(value: F::Value) -> F::Value {
    <F::Value as Convert<F::Value>>::convert(value)
}

fn from_count<F: Family>(value: u32) -> F::Value {
    <F::Value as Convert<u32>>::convert(value)
}

struct Concrete;

impl Family for Concrete {
    type Value = Count;
}

fn main() {
    let value = from_count::<Concrete>(18);
    if identity::<Concrete>(value).0 != 19 {
        loop {}
    }
}
