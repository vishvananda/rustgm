struct Tail<T:?Sized>{tag:u8,tail:T}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let mut x=Tail{tag:7,tail:[3u8,5]};let r=&mut x;let s=r as &mut Tail<[u8]>;r.tail[0]=9;check(s.tail[0]==3);}
