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

    fn route(value: &'rows u32) -> &'index u32 {
        Self::shorten(value)
    }
}

fn fail(zero: u32) -> u32 {
    1 / zero
}

fn main() {
    let value = 7;
    let header = Header { value: &value };
    let domain = Domain { header: &header };
    let result = Domain::route(domain.header.value);
    if *result != 7 {
        let _ = fail(0);
    }
}
