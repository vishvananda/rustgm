fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<T>(&self,x:T){let _a=x;let _b=x;}}fn main(){}
