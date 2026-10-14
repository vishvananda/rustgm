fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum E<T>{A(T),B(T,T),C}fn main(){let f:fn(u16,u16)->E<u16>=E::B;let a=f(300,400);check(match a{E::B(x,y)=>x+y==700,_=>false});let b=E::A(7u8);check(match b{E::A(x)=>x==7,_=>false});}
