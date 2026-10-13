fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:u64)->u64{x+1}#[derive(Clone,Copy)]#[repr(C,packed)]struct P{a:u8,f:fn(u64)->u64}static A:[P;3]=[P{a:7,f:inc};3];fn main(){check((A[0].f)(1)==2 && (A[1].f)(4)==5 && (A[2].f)(8)==9 && A[1].a==7);}
