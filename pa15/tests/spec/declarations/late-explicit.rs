fn id<'a>(x:&'a u8)->&'a u8{x}fn f(){id::<'static>(&7);}
fn main() {}
