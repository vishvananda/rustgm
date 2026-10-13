fn f(s:&mut [u8]){if let [a,..,b]=s{let c=&mut s[1];*a+=*c;*b+=1;}}
fn main() {}
