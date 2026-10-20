trait A{fn f<'a,T>(x:&'a ())->impl Sized+'a;}impl A for u8{fn f<'a,T>(x:&'a ())->&'a T{loop{}}}fn main(){}
