fn check(ok:bool){if !ok{let _=failure(0);}} fn failure(x:i32)->i32{1/x}
#[repr(align(64))]struct S(u64,u8); fn id(x:S)->S{x} fn main(){let s=id(S(42,8));check(s.0==42 && s.1==8);}
