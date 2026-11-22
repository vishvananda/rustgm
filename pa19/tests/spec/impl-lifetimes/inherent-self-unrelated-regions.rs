struct Domain<'first, 'second> {
    first: &'first u32,
    second: &'second u32,
}

impl<'first, 'second> Domain<'first, 'second> {
    fn shorten(value: &'second u32) -> &'first u32 {
        value
    }
}

fn main() {}
