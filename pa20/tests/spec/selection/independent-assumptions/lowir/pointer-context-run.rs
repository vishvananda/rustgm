fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out;}impl A for u8{type Out=u32;}trait B<V>{fn m()->V;}impl B<u8>for u16{fn m()->u8{7}}impl B<u32>for u16{fn m()->u32{43}}fn f<T:A,U:B<<T as A>::Out>+B<u8>>()->u8{let m:fn()->u8=U::m;m()}fn main(){check(f::<u8,u16>()==7);}
