fn f<'a>(p:fn(&'a u8)->&'a u8){let _:for<'b>fn(&'b u8)->&'b u8=p;}
fn main() {}
