static mut SLOT:u8=7;static DATA:&mut u8=unsafe{&mut SLOT};fn main(){if *DATA!=7{let x=0u8;let _=1/x;}}
