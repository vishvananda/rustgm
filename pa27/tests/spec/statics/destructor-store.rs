static mut N:u8=0;struct S(u8);impl Drop for S{fn drop(&mut self){unsafe{N+=1;}}}static mut X:S=S(3);fn f(){unsafe{X=S(9);}}fn main(){}
