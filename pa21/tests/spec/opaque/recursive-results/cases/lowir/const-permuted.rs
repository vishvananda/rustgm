fn f<const N:usize,const M:usize>(n:u8)->impl Copy{if n==0{1u8}else{f::<M,N>(n-1)}}fn main(){}
