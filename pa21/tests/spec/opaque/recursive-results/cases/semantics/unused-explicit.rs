fn f<'a>(n:u8,x:&'a u8)->impl Copy+use<>{if n==0{1u8}else{let y=2;f(n-1,&y)}}fn main(){}
