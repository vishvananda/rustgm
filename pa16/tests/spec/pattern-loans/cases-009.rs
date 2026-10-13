fn f(s:&mut [u8]){if let [..,b]=s{let c=&mut s[0];*b+=*c;}}
fn main() {}
