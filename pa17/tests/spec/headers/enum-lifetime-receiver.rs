enum S<'a>{A(&'a mut u8),B}impl Drop for S<'_>{fn drop(&mut self){match self{S::A(n)=>**n+=1,S::B=>()}}}fn main(){}
