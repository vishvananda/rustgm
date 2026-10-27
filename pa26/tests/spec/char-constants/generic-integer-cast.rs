fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C<const V:usize>;fn take(_:C<{char::MAX as usize}>)->u8{7}fn main(){check(take(C::<1114111>)==7);}
