fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn main(){let x:Tail<[u8;0]>=Tail{tag:7,tail:[]};let s=&x as &Tail<[u8]>;check(s.tag==7);check(match s.tail{[]=>true,_=>false});}
