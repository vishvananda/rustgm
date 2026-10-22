fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:usize>;impl<const N:usize>S<N>{const C:u8={let n=7;n};}fn run<const N:usize>(n:u8)->bool{match n{S::<N>::C=>true,_=>false}}fn main(){check(run::<2>(7));check(!run::<5>(8));}
