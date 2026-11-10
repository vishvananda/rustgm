fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
struct S(u8);fn main(){let x=Tail{tag:7,tail:[S(3),S(5)]};let s=&x as &Tail<[S]>;check(s.tail[1].0==5);check(x.tail[0].0==3);}
