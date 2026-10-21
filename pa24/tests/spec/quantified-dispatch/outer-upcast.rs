fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<'a>(x:&'a u8)->u8{let f=move|_:&u8|x;let p:&dyn for<'b>Fn(&'b u8)->&'a u8=&f;let q:&dyn for<'b>FnMut(&'b u8)->&'a u8=p;let _=q;*p(&0)}fn main(){let x=7;check(run(&x)==7);}
