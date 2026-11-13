pub fn f<'b,'c>(x:&'c &'b u8)->&'static u8{(|&s|s)(x)}
fn main(){}
