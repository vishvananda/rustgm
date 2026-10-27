fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C<const V:usize>;fn main(){let _=C::<{char::MAX}>;}
