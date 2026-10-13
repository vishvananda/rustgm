fn g<'a:'a,'b:'b>(_:&'a u8)->&'b u8{loop{}}fn f<'a>(){let mut p=g::<'a,'a>;p=g::<'static,'a>;let _:fn(&'static u8)->&'a u8=p;}
fn main() {}
