fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum R{A=0,B=*(&S{e:E::B}.e) as isize+1}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}
#[repr(packed(8))]struct S{e:E}
static V:S=S{e:E::B};
fn main(){check(R::B as isize==4294967297);check(V.e as u64==4294967296);}
