fn f<'a>(n:u8,x:&'a u8)->impl Copy+'a{if n==0{x}else{let y=2;f(n-1,&y)}}fn main(){}
