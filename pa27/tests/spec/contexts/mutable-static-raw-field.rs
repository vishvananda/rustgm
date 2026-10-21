struct S{x:u8}static mut X:S=S{x:7};fn f()->*const u8{&raw const X.x}fn main(){}
