struct S{x:u8}static mut X:S=S{x:0};fn f(){unsafe{let S{x:ref r}=X;let _=r;}}fn main(){}
