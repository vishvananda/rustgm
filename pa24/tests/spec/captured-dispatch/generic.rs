fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn use_it<const N:u8>(x:u8)->u8{let f=move|n:u8|x+N+n;let p:&dyn Fn(u8)->u8=&f;p(2)}fn main(){check(use_it::<3>(7)==12);check(use_it::<4>(7)==13);}
