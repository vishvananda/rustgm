fn f<T:Copy,U:Copy>(n:u8,x:T,y:U)->impl Copy{if n==0{x}else{f(n-1,y,x)}}fn main(){}
