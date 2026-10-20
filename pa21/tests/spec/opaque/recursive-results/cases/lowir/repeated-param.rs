fn f<T:Copy,U:Copy>(n:u8,x:T,y:U)->impl Copy{if n==0{1u8}else{f(n-1,x,x)}}fn main(){}
