fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn run<F:Fn()->u8>(f:F)->u8{f()}fn main(){let mut n=7;let r=&mut n;let f=move||{let _r=r;7};let _=run(f);}
