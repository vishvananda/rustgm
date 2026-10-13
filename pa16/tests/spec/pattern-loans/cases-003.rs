fn f(s:&mut [u8]){if let [a,rest @ ..,z]=s{if let [b,inner @ ..,y]=rest{*a+=*b;inner[0]+=1;*z+=*y;}}}
fn main() {}
