static mut X:u8=0;fn f(){unsafe{let ref r=X else{return;};let _=r;}}fn main(){}
