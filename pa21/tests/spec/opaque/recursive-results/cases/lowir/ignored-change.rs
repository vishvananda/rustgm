fn f<T:Copy>(n:u8,x:T)->impl Copy{if n!=0{let _=f(n-1,true);}x}fn main(){}
