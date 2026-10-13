fn f(s:&mut [u8]){if let [a,rest @ ..]=s{let b=&mut s[0];*a+=*b;let _=rest;}}
fn main() {}
