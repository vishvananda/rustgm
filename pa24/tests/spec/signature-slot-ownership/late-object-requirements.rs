fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Apply:for<'a>Fn(&'a u8)->u8{}impl<F:for<'a>Fn(&'a u8)->u8>Apply for F{}fn run(f:&dyn Apply)->u8{f(&0)}fn main(){let n=7;let a=|_:&u8|n;let first:&dyn Apply=&a;let second:&dyn Apply=&a;check(first(&0)==7);check(second(&0)==7);}
