enum R{A=0,B=(&S{e:[]}.e,1).1}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}
#[repr(packed(4))]struct S{e:[E;0]}
fn main(){}
