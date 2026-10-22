fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick{type Out;const C:Self::Out;}struct S<'a,'b>(&'a u8,&'b u8);impl<'a:'b,'b>Pick for S<'a,'b>{type Out=&'b u8;const C:Self::Out={let n:&'a u8=&7;n};}fn f<'a:'b,'b>(_:&'a u8,_:&'b u8,x:&'b u8)->bool{match x{<S<'a,'b>as Pick>::C=>true,_=>false}}fn main(){let a=7;let b=8;check(f(&a,&b,&7));check(!f(&a,&b,&8));}
