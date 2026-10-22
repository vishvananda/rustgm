const N:u32={let mut a=[0u32;4];let mut i=0;while i<4{let p=&raw mut a[i];unsafe{*p=i as u32+10}i+=1;}a[0]+a[1]+a[2]+a[3]};
fn main(){check(N==46)}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
