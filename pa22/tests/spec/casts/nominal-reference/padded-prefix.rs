fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C)]struct Tail<T:?Sized>{tag:u8,tail:T}
fn main(){let x=Tail{tag:7,tail:[9u64,23]};let s=&x as &Tail<[u64]>;check(s.tag==7);check(s.tail[1]==23);check(match s.tail{[_,_]=>true,_=>false});}
