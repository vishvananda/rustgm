fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn make(n:&mut u8)->&'static Tail<[u8;2]>{*n+=1;&Tail{tag:7,tail:[3,5]}}fn main(){let mut n=0;let s=make(&mut n) as &Tail<[u8]>;check(n==1);check(s.tag==7);check(s.tail[1]==5);check(match s.tail{[_,_]=>true,_=>false});}
