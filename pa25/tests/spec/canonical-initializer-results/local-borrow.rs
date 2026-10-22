fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8={let n=7;let r=&n;*r};}fn main(){check(match 7{S::<'static>::C=>true,_=>false});check(!match 8{S::<'static>::C=>true,_=>false});}
