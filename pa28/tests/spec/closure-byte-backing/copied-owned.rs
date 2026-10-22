const fn transfer<T:Copy>(x:T)->T{unsafe{let source=x;let mut target=x;core::ptr::copy_nonoverlapping(&raw const source,&raw mut target,1);target}}fn main(){let f=const{let x=37u32;transfer(move||x)};check(f()==37)}
fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
