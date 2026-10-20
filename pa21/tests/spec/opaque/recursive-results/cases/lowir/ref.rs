fn f<'a>(n:u8,x:&'a u8)->impl Copy+'a{if n==0{x}else{f(n-1,x)}}fn main(){}
