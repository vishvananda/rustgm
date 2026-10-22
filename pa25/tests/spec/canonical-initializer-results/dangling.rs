fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a>S<'a>{const C:&'a u8={let n=7;&n};}fn main(){let _=match &7{S::<'static>::C=>true,_=>false};}
