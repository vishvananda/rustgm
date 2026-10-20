fn f<T:Copy>(n:u8,x:T)->impl Copy{if n==0{x}else{f(n-1,x)}}fn main(){}
