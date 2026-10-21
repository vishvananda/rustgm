static mut X:u8=0;#[allow(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}fn main(){}
