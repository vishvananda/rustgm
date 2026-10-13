fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:&mut [u8]){x[1]+=1}fn main(){let mut a=[1,41,3];inc(&mut a);check(a[1]==42);}
