fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast<'a,'b>(x:&'b Tail<[&'a u8;1]>)->&'b Tail<[&'b u8]>where 'a:'b{x as &'b Tail<[&'b u8]>}fn main(){let v=5u8;let x=Tail{tag:7,tail:[&v]};let s=cast(&x);check(*s.tail[0]==5);}
