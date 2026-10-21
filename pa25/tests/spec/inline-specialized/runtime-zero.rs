fn touch(n:&mut u8)->u8{*n+=1;7}fn f<const N:usize>(n:&mut u8)->u8{let _=[touch(n);N];7}pub fn run()->bool{let mut n=0;let _=f::<0>(&mut n);let a=n==1;let _=f::<2>(&mut n);a&&n==2}
