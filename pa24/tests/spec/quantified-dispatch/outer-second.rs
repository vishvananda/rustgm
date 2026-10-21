fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<'a,'c>(x:&'a u8,y:&'c u8)->u8{let f=move|_:&u8|y;let p:&dyn for<'b>Fn(&'b u8)->&'c u8=&f;*p(x)}fn main(){let x=7;let y=9;check(run(&x,&y)==9);}
