fn f<const N:usize>(n:u8)->impl Copy{if n==0{1u8}else{f::<3>(n-1)}}fn main(){}
