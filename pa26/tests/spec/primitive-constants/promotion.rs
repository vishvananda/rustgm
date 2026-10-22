fn check(b:bool){if !b{let z=0;let _=1/z;}}fn value()->&'static u32{&u32::MAX}fn main(){check(*value()==4294967295u32);}
