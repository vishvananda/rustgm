const fn pass(p:*mut u8)->*mut u8{p}static mut SLOT:u8=7;static DATA:&mut u8=unsafe{&mut *pass(&raw mut SLOT)};fn main(){if *DATA!=7{let x=0u8;let _=1/x;}}
