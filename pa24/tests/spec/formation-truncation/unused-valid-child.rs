fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn make()->impl Fn()->u8{let good=||{let mut n=7u8;let p=&mut n;*p=8;*p};||7}fn main(){}