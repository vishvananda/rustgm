#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:F::A}.e) as u8+1}
#[repr(u8)]#[derive(Copy,Clone)]enum F{A=0,B=*(&T{e:E::A}.e) as u8+2}
#[repr(packed)]struct S{e:F}
#[repr(packed)]struct T{e:E}
fn main(){}
