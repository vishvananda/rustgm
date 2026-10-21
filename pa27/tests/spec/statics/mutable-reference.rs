static mut X:u8=0;fn f()->&'static mut u8{unsafe{&mut X}}fn main(){}
