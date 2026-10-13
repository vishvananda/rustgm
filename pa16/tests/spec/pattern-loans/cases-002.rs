fn f(s:&mut [u8]){if let [a,rest @ ..]=s{let b=&mut rest[0];*a+=*b;}}
fn main() {}
