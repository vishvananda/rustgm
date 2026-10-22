fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn main(){let a=[1u8,2,3];check(a.len()==3);let a:[u8;0]=[];check(a.len()==0);}
