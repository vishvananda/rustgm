fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8){let i=id;let _a=i(x);let _b=i(y);}
fn main() {}
