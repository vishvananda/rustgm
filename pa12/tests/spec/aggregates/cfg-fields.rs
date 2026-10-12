fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
#[cfg_attr(all(),repr(C))]struct S{#[cfg(any())]missing:Unknown,a:u64}fn main(){let s=S{a:7};check(s.a==7);}
