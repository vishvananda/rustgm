fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
trait Query {
    type Value;
}
struct Surrounding(u8);
impl Query for Surrounding {
    type Value = bool;
}
// The projection is stored only behind a pointer to a sized value, as a
// vector stores its element type, so the walk over by-value fields meets none.
struct Raw<T> {
    first: *const T,
    len: usize,
}
struct Occurrences<Q: Query> {
    query: Q,
    raw: Raw<Q::Value>,
    direct: *const Q::Value,
}
struct Holder {
    occurrences: Occurrences<Surrounding>,
    count: u32,
}
fn main() {
    let flag = true;
    let h = Holder {
        occurrences: Occurrences {
            query: Surrounding(4),
            raw: Raw { first: &flag, len: 3 },
            direct: &flag,
        },
        count: 2,
    };
    check(h.count == 2 && h.occurrences.query.0 == 4 && h.occurrences.raw.len == 3);
    check(unsafe { *h.occurrences.raw.first && *h.occurrences.direct });
}
