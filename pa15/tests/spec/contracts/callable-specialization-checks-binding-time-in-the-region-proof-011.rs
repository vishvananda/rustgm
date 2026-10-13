fn f<'a:'b,'b>(p:fn(&'a u8)->&'b u8){let _:fn(&'b u8)->&'a u8=p;}
fn main() {}
