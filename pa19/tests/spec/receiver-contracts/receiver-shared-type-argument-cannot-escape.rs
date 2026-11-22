enum Choice<T> {
    Supplied(T),
    Missing,
}

impl<T> Choice<T> {
    fn choose(self, fallback: T) -> T {
        match self {
            Self::Supplied(value) => value,
            Self::Missing => fallback,
        }
    }
}

struct Scope {
    seen: u8,
}

fn invalid(frame: &mut Scope, shared: Choice<&'static mut u8>) -> &'static mut u8 {
    let Scope { seen: own } = frame;
    shared.choose(own)
}

fn main() {}
