fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T>S<T>{const C:u8={let n=7;n};}fn run<'a>(_:&'a u8)->u8{S::<&'a u8>::C}fn main(){let n=7;check(run(&n)==7);}
