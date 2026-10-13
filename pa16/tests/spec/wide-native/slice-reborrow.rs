fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn inc(x:&mut [u8]){x[0]+=1}fn main(){let mut a=[40];let s:&mut [u8]=&mut a;inc(s);inc(s);check(s[0]==42);}
