fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn once<F:FnOnce()->R,R>(f:F)->R{f()}fn twice<F:FnMut()->R,R>(mut f:F)->R{let _=f();f()}trait Pick{type Out:Copy;}impl<'a> Pick for &'a u8{type Out=&'a u8;}fn take<T:Pick>(x:T::Out)->T::Out{let f=move||x;once(f)}fn main(){let n=7;check(*take::<&u8>(&n)==7);}
