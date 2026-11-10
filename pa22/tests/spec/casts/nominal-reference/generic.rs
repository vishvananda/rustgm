fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a,T,const N:usize>(x:&'a Tail<[T;N]>)->&'a Tail<[T]>{x as &'a Tail<[T]>}fn main(){let x=Tail{tag:7,tail:[3u8,5]};let s=cast(&x);check(s.tag==7);check(s.tail[1]==5);check(match s.tail{[_,_]=>true,_=>false});}
