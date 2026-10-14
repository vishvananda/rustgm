fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
enum R{A=0,B=(&S{w:W{e:(E::B,[])}}.w).e.0 as isize+1}
#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}
#[derive(Copy,Clone)]struct W{e:(E,[u16;0])}
#[repr(packed(8))]struct S{w:W}
fn main(){check(R::B as isize==4294967297);let s=S{w:W{e:(E::B,[])}};check(s.w.e.0 as u64==4294967296);}
