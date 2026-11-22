struct Header<'rows> {
    value: &'rows u32,
}

struct Domain<'index, 'rows> {
    header: &'index Header<'rows>,
}

impl<'index, 'rows> Domain<'index, 'rows> {
    fn shorten(value: &'rows u32) -> &'index u32 {
        value
    }
}

fn escape<'short>(value: &'short u32) -> &'static u32 {
    Domain::<'static, 'short>::shorten(value)
}

fn main() {}
