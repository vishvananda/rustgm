fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}struct S;unsafe impl A for S{}fn main(){}
