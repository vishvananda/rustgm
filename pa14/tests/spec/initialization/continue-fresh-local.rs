fn f(mut n:u8)->u8{let mut sum=0;'outer:while n>0{let x;x=n;n-=1;if x==2{continue 'outer}sum+=x;}sum}
