struct S{a:u8,b:u8}fn f(s:&mut [S]){if let [..,S{a,b}]=s{*a+=*b;}}
fn main() {}
