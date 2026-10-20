fn f(x:&(dyn Send+Sync))->&dyn Send{x}
fn main(){}
