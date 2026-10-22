fn operate()->u32{let mut x=0x00070001u32;unsafe{(*(&raw mut x as *mut [u8;4]))[2]=11;}x}fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn main(){check(operate()==0x000b0001);}
