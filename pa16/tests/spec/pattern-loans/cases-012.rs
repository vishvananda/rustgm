fn f(s:&[u8]){if let [a,rest @ ..]=s{rest[0]=*a;}}
fn main() {}
