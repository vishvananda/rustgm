struct S{a:u8}fn g()->S{S{a:2}}fn f()->&'static S{&S{a:1,..g()}}
fn main() {}
