fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make<T:Copy>{fn make<const N:usize>(x:T)->impl Copy;}impl<T:Copy>Make<T> for u8{fn make<const N:usize>(x:T)->[T;N]{[x;N]}}fn main(){let a:[u16;3]=<u8 as Make<u16>>::make::<3>(61);check(a[2]==61);}
