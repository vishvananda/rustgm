trait A<'a,'b>{type Out;}fn f<'x,'y,T>(x:&'x u8,y:&'y u8)-><T as A<'x,'y>>::Out where for<'a,'b>T:A<'a,'b,Out=(&'a u8,&'b u8)>{(y,x)}fn main(){}
