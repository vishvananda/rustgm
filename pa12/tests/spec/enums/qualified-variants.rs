fn fail(x: u8) -> u8 { 1 / x }
fn check(ok: bool) { if !ok { let _ = fail(0); } }
enum E { Unit, Tuple(u8) }
const C: E = <E>::Unit;
fn main() {
    let f: fn(u8) -> E = <E>::Tuple;
    match f(7) { E::Tuple(x) => check(x == 7), _ => check(false) }
    match C { E::Unit => (), _ => check(false) }
}
