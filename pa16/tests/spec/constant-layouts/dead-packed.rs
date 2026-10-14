enum R{A=0,B=if true{1}else{*(&S{e:E::B}.e) as isize}}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}
#[repr(packed(4))]struct S{e:E}
fn main(){}
