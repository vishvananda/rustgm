fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a> for Tag{type Out=&'a u8;const V:Self::Out=&7;}struct S<'a,'b,T>(&'a u8,&'b u8,T);impl<'a:'b,'b,T:Pick<'b>>S<'a,'b,T>{const C:T::Out={let n=T::V;n};}fn f<'a,'b>(_:&'a u8,_:&'b u8,x:&'b u8)->bool{match x{S::<'a,'b,Tag>::C=>true,_=>false}}fn main(){}
