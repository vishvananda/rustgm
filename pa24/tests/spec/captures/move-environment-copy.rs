fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let x=7u8;let f=move ||x+2;let g=f;check(f()==9&&g()==9);}
