fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'b>(x:&'b u8)->&'b u8{id::<'b>(x)}
fn main() {}
