fn f(n:u8)->impl Copy{if n==0{1u8}else{g(n-1)}}fn g(n:u8)->impl Copy{f(n)}fn main(){}
