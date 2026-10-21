fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn use_it<const N:u8>(x:u8)->u8{let mut f=move|n:u8|x+N+n;let p:&mut dyn FnMut(u8)->u8=&mut f;p(2)}fn main(){check(use_it::<3>(7)==12);check(use_it::<4>(7)==13);}
