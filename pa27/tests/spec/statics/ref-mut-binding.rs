static mut X:u8=0;fn f(){unsafe{let ref mut r=X;let _=r;}}fn main(){}
