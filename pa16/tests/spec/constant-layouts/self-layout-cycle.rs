#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:E::A}.e) as u8+1}
#[repr(packed)]struct S{e:E}
fn main(){}
