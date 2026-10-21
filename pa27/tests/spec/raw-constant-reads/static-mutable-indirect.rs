static mut S:u32=37;const fn pointer()->*const u32{&raw const S}const X:u32=unsafe{*pointer()};fn main(){let _=X;}
