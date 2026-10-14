fn f<'a,T>(x:&'a T)->&'a T{let y=x;&*y}
fn main(){}
