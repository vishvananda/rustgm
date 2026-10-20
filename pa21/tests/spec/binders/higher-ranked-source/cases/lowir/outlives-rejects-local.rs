fn f<T>(x:T)where for<'a>T:'a{}fn run<'x>(x:&'x u8){f(x);}fn main(){let n=1;run(&n);}
