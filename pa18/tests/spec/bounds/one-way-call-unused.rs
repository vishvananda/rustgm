fn duplicate<V: Copy>(value: V) -> (V, V) { (value, value) }
fn f<'x: 'y, 'y, T>(x: (&'x u8, &'y u8, T)) -> ((&'x u8, &'y u8, T), (&'x u8, &'y u8, T))
where (&'x u8, &'x u8, T): Copy
{ duplicate(x) }
fn main() {}
