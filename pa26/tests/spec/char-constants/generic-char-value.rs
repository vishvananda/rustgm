fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C<const V:char>;fn take(_:C<{char::MAX}>)->u8{7}fn main(){check(take(C::<'\u{10ffff}'>)==7);}
