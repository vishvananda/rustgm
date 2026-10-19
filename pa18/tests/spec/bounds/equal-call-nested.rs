fn duplicate<V: Copy>(value: V) -> (V, V) { (value, value) }
fn f<'x: 'y, 'y: 'x, T>(x: ((&'x u8, &'y u8, T), u8)) -> (((&'x u8, &'y u8, T), u8), ((&'x u8, &'y u8, T), u8))
where (&'x u8, &'x u8, T): Copy
{ duplicate(x) }
static A: u8 = 3;
static B: u8 = 5;
fn main() {
    let proof = &B;
    let p = f(((&A, &B, 7u8), 9u8));
    if *p.0.0.0 != 3 || *p.1.0.1 != 5 || p.1.0.2 != 7 || p.0.1 != 9 { loop {} }
}
