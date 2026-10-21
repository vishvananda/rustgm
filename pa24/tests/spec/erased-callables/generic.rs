fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make<T:Copy>(x:T){let f=move||x;let g:&dyn Fn()->T=&f;g();} fn main(){make(7u8);}
