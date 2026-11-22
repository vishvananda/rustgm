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

fn write(frame: &mut Scope, shared: Choice<&mut u8>) {
    let Scope { seen: own } = frame;
    let seen = shared.choose(own);
    *seen = 9;
}

fn experiment() -> i32 {
    let mut frame = Scope { seen: 1 };
    let mut external = 2;
    write(&mut frame, Choice::Supplied(&mut external));
    if frame.seen != 1 || external != 9 {
        return 1;
    }
    write(&mut frame, Choice::Missing);
    if frame.seen == 9 { 0 } else { 1 }
}

fn fail(zero: u8) -> u8 {
    1 / zero
}

fn main() {
    if experiment() != 0 {
        let _ = fail(0);
    }
}
