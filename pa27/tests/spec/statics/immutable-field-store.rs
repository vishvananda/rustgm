struct S{x:u8}static X:S=S{x:0};fn f(){unsafe{X.x=7;}}fn main(){}
