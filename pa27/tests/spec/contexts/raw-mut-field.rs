struct S{x:u8}static mut X:S=S{x:0};fn f()->*mut u8{unsafe{&raw mut X.x}}fn main(){}
