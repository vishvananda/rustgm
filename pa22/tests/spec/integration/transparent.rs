fn check(ok: bool) {
    if !ok {
        let _ = fail(0);
    }
}
fn fail(x: u8) -> u8 {
    1 / x
}
#[repr(transparent)]
struct W<T: ?Sized> {
    z: (),
    tail: T,
}
fn read(w: &W<[u16]>) -> u16 {
    w.tail[1]
}
fn main() {
    let w = W {
        z: (),
        tail: [3u16, 7],
    };
    check(read(&w) == 7);
}
