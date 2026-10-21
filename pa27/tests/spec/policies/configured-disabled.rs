static mut X:u8=0;#[cfg_attr(any(),allow(static_mut_refs))]fn f()->&'static u8{unsafe{&X}}fn main(){}
