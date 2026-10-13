struct N(u8);fn take(_:N)->bool{true}fn entry()->u64{let x=N(7);match x{n if take(n)=>1,_=>2}}
fn main(){}