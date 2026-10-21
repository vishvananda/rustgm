fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<'a,'c>(x:&'a u8,y:&'c u8)->&'c u8{let f=move|_:&u8|x;let p:&dyn for<'b>Fn(&'b u8)->&'c u8=&f;p(y)}fn main(){}
