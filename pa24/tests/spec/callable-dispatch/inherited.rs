fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait More:Fn(u8)->u8{}impl<T:Fn(u8)->u8>More for T{}fn inc(x:u8)->u8{x+2}fn main(){let p:&dyn More=&inc;check(p(7)==9);}
