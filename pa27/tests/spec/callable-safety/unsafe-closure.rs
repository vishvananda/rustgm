fn main(){let f:unsafe fn(&u8)->&u8=|x|x;let a=7;let _=unsafe{f(&a)};}
