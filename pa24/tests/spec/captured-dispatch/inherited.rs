fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Op:Fn(u8)->u8{}impl<T:Fn(u8)->u8>Op for T{}fn main(){let x=7u8;let f=move|n:u8|x+n;let p:&dyn Op=&f;check(p(2)==9);}
