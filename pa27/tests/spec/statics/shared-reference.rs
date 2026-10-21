static mut X:u8=0;fn f()->&'static u8{unsafe{&X}}fn main(){}
