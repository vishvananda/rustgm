fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T,U>(T,U);impl<T:Copy,U>S<T,U>{const C:u8={let n=7;n};}fn run<T:Copy,U>(n:u8)->bool{match n{S::<T,U>::C=>true,_=>false}}fn main(){check(run::<u8,u16>(7));check(!run::<u32,u64>(8));}
