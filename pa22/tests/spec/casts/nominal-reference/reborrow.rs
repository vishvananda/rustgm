fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn main(){let mut x=Tail{tag:7,tail:[3u8,5]};let r=&mut x;let s=r as &mut Tail<[u8]>;s.tail[0]=8;r.tail[1]=9;check(x.tail[0]==8&&x.tail[1]==9);}
