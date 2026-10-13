struct S<'a>{value:&'a u8}fn f<'a,'b>(_:&'a u8,S{value:x}:S<'b>)->&'b u8{x}
fn main() {}
