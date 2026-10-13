fn f<'b:'static>(p:for<'a>fn(&'a &'b u8,&'a u8)->&'a u8){let q:for<'a>fn(&'a &'b u8,&'b u8)->&'a u8=p;}
fn main() {}
