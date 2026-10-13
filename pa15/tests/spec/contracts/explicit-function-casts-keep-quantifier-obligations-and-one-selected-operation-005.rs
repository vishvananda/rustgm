fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let p=id::<'static> as fn(&'static u8)->&'static u8;}
fn main() {}
