struct S{x:u8}static mut X:S=S{x:0};fn f()->&'static u8{unsafe{&X.x}}fn main(){}
