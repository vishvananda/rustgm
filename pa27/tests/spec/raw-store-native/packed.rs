#[repr(packed)]struct P{a:u8,b:u32}#[repr(packed)]struct O{a:u8,p:P}
const A:u32={let mut p=P{a:1,b:7};let r=&raw mut p;unsafe{(*r).b=37;}p.b};
const B:u32={let mut o=O{a:1,p:P{a:2,b:7}};let r=&raw mut o;unsafe{(*r).p.b=41;}o.p.b};
fn main(){check(A==37);check(B==41);}
