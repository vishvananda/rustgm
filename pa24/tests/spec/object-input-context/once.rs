fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:FnOnce(u8)->u8{}impl<F:FnOnce(u8)->u8>Apply for F{}fn main(){let n=4;let f=move|x|x+n;let _o:&dyn Apply=&f;check(f(3)==7);}