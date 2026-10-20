fn f(x:&dyn Send)->&(dyn Send+Sync){x}
fn main(){}
