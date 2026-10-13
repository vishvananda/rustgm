fn f<'a>()->&'a u8{&7}fn g()->&'static u8{f::<'static>()}
fn main() {}
