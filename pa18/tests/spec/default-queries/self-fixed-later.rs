type A<T=A<u8>,const N:usize=3>=(T,[u8;N]);fn f(x:A)->((u8,[u8;3]),[u8;3]){x}
fn main(){}
