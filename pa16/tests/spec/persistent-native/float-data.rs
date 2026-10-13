fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
static A:(f64,f32,f64)=(-0.0,0.0/0.0,1.0/0.0);fn main(){check(1.0/A.0 == -A.2 && A.1!=A.1 && A.2>1.0);}
