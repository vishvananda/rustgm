fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick{type Out:Copy;}impl Pick for u8{type Out=u8;}fn run<T:Pick>(x:T::Out)->u8{let f=move|a:&u8|{let y=x;let _=y;*a};let p:&dyn Fn(&u8)->u8=&f;p(&7)}fn main(){check(run::<u8>(3)==7);}
