fn f(s:&mut [u8]){match s{[a,rest @ ..] if {rest[0]=1;true}=>*a=2,_=>()}}
fn main() {}
