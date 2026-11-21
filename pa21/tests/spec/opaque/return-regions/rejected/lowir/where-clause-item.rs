trait Take<T>{fn take(&self,t:T)->u8;}struct A;impl<'x>Take<&'x u8>for A{fn take(&self,t:&'x u8)->u8{*t}}
fn run<X>(x:X)->u8 where X:Take<&u8>{let v=7;x.take(&v)}fn main(){let _=run(A);}
