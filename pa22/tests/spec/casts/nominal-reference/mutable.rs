fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Tail<T:?Sized>{tag:u8,tail:T}
fn cast(x:&mut Tail<[u8;2]>)->&mut Tail<[u8]>{x as &mut Tail<[u8]>}fn main(){let mut x=Tail{tag:7,tail:[3u8,5]};let s=cast(&mut x);s.tail[1]=9;check(s.tag==7);check(x.tail[1]==9);}
