macro_rules! id{($e:expr)=>{$e}}fn f()->&'static u8{&(7u8/id!(2))}
fn main() {}
