fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<'a>(x:&'a u8)->u8{let f=move||*x;let p:&dyn Fn()->u8=&f;p()}fn main(){let x=3u8;check(run(&x)==3);{let y=4u8;check(run(&y)==4);}check(run(&x)==3);}
