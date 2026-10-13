fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static N:u64=42;#[repr(C,packed)]struct P{tag:u8,r:&'static u64}static A:P=P{tag:7,r:&N};fn main(){let r=A.r;check(*r==42 && A.tag==7);}
