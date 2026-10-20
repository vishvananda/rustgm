fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait F{type Out<const N:usize>;fn make<const N:usize>(x:[u8;N])->Self::Out<N>;}struct S;impl F for S{type Out<const N:usize>=[u8;N];fn make<const N:usize>(x:[u8;N])->Self::Out<N>{x}}fn main(){let x=S::make([3u8,7,11]);check(x[1]==7);}
