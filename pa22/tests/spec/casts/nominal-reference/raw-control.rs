fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn main(){let x=Tail{tag:7,tail:[3u8,5]};let s=&x as &Tail<[u8]>;let p=s as *const Tail<[u8]>;let q=&x as *const Tail<[u8]>;check(p==q);check(s.tag==7);check(s.tail[1]==5);check(match s.tail{[_,_]=>true,_=>false});}
