fn check(ok:bool){if !ok{let _=failure(0);}}
fn failure(x:i32)->i32{1/x}
fn main(){let mut p=(1u8,(2u16,3u32));p.1.0=77;let q=p;p.1.1=88;check(q.1.0==77 && q.1.1==3 && p.1.1==88);}
