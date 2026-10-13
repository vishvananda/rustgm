fn f<'a>(p:for<'b>fn(&'b u8)->&'b u8){let _:fn(&'a u8)->&'a u8=p;}
fn main() {}
