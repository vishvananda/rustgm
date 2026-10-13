fn f<'a:'b,'b>(p:fn(&'b u8)->&'a u8){let _:fn(&'a u8)->&'b u8=p;}
fn main() {}
