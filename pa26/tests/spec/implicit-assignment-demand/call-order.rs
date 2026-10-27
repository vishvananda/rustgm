fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn left<'a>(x:&'a mut u32,t:&mut u32)->&'a mut u32{*t=*t*10+1;x}fn right<'a>(y:&'a u32,t:&mut u32)->&'a u32{*t=*t*10+2;y}fn main(){let mut x=3u32;let y=4u32;let mut t=0u32;*left(&mut x,&mut t)+=right(&y,&mut t);check(x==7&&t==12);}
