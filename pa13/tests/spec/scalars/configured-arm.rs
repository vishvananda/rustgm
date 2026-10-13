fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:bool)->u8{match x{#[cfg(no)] _=>unknown,true=>1,false=>2}}fn main(){check(f(false)==2);}