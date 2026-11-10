fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
struct Outer<T:?Sized>{prefix:u16,inner:Tail<T>}fn cast(x:&Outer<[u8;2]>)->&Outer<[u8]>{x as &Outer<[u8]>}fn main(){let x=Outer{prefix:300,inner:Tail{tag:7,tail:[3u8,5]}};let s=cast(&x);check(s.prefix==300);check(s.inner.tag==7);check(s.inner.tail[1]==5);check(match s.inner.tail{[_,_]=>true,_=>false});}
