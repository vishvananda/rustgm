fn id(x:&u8)->&u8{x}fn value()->u8{1}fn f(){let p=id(&value());let y=*p;}
fn main() {}
