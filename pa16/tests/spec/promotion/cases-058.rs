struct S{a:[u8;2],b:(u8,u8)}fn f()->&'static S{&S{a:[3;2],b:(4,5)}}
fn main() {}
