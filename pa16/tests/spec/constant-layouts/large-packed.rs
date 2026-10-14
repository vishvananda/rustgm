enum R{A=0,B=*(&S{e:E::B}.e) as isize+1}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}
#[repr(packed(4))]struct S{e:E}
fn main(){}
