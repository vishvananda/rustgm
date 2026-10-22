fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let r=&mut u32::MAX;*r=3;check(*r==3);check(u32::MAX==4294967295u32);}
