fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
fn f(x:char)->u8{match x{'퟿'..''=>1,_=>2}}fn main(){check(f('퟿')==1&&f('')==2);}