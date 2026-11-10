fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn main(){let x=Tail{tag:7,tail:[();3]};let s=&x as &Tail<[()]>;check(s.tag==7);check(match s.tail{[_,_,_]=>true,_=>false});}
