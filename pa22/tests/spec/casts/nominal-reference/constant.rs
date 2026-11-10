fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
const fn cast(x:&Tail<[u8;2]>)->&Tail<[u8]>{x as &Tail<[u8]>}const X:Tail<[u8;2]>=Tail{tag:7,tail:[3,5]};const V:&Tail<[u8]>=cast(&X);fn main(){let s=V;check(s.tag==7);check(s.tail[1]==5);check(match s.tail{[_,_]=>true,_=>false});}
